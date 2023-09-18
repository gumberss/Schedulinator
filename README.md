# Schedulinator

The core objective of this project is to develop a proof of concept for a high-performance scheduler capable of efficiently processing millions of tasks per second, leveraging the power of Redis Sorted Sets. This initiative seeks to identify the next tasks directly through Redis, thus eliminating the need for expensive database reads. Before using this project, it's crucial to conduct a thorough analysis to determine if it aligns with your specific needs. For instance, if your execution rate is relatively low, opting for Redis may not be the most cost-effective choice. Redis incurs costs based on the number of nodes (when using AWS at least), so in such cases, using a database like DynamoDb directly could be a more economical alternative.

### Sorted Set with Timestamp

By utilizing the Unix Timestamp or another integer-based date representation as the score within the sorted set, you can precisely determine when a task is scheduled to run. Tasks are stored in ascending order based on their scores, meaning that tasks with lower scores should be executed first, while tasks with higher scores indicate a later execution time. Let's illustrate this concept with an example:

``` 
zadd schedules 1693964754 task1 // 13 August 2023 22:12:34
zadd schedules 1693965795 task2 // 6 September 2023 02:03:15
zadd schedules 1693974803 task3 // 6 September 2023 04:33:23
zadd schedules 1694064809 task4 // 7 September 2023 05:33:29
zadd schedules 1704064809 task5 // 31 December 2023 23:20:09
```
We are able to get the tasks to execute in order with the command:

```zrange schedules 0 -1```

That yields the result:
```
1) "task1"
2) "task2"
3) "task3"
4) "task4"	 
5) "task5"
```

And that's not all; you can also retrieve only the schedules that should have been executed up to the present moment. Consider today's date as September 7, 2023, at 00:00:00 (Unix Timestamp: 1694044800). In this scenario, you can obtain the range of tasks that should have been executed by now using the following command:

```zrangebyscore schedules 0 1694044800 withscores```

That yields the result:

```
1) "task1"
2) 1693964754.0
3) "task2"
4) 1693965795.0
5) "task3"
6) 1693974803.0
```
Note that only the first three elements in the schedules Sorted Set were returned

### Idempotency

To address a potential corner case where the service might experience a failure right after task execution but before updating the Sorted Set, resulting in outdated data within Redis, there is no foolproof solution. This situation resembles the Two Generals' problem, making it challenging to ensure absolute synchronization. In a similar scenario, consider when the scheduler sends a request to a service, and the service begins processing it. However, before the service can return a response, the scheduler experiences a timeout. In this case, the task may end up being reprocessed.

<figure class="image">
  <img src="https://github.com/gumberss/Schedulinator/assets/38296002/2d810b13-f71d-4150-80fe-8a45dc3e4367" alt="The service went down before updating the score">
  <figcaption>The service went down before updating the score</figcaption>
</figure>
 <br/><br/>
<figure class="image">
  <img src="https://github.com/gumberss/Schedulinator/assets/38296002/d489fc74-c1dd-4277-ac1e-7f3eda5e5db2" alt="The requester service received a timeout, but the receiver service processed the first request">
  <figcaption>The requester service received a timeout, but the receiver service processed the first request</figcaption>
</figure>
 <br/><br/>

One possible approach to mitigate the other service timeout issue is to always send a request to the endpoint with a unique identifier generated deterministically based on the score of the task within the Sorted Set. By doing so, you can rely on the receiving service to leverage idempotency to handle the problem. This way, even if the data within Redis becomes temporarily out of date due to a server failure, the unique identifier associated with each task can help ensure that duplicate requests are appropriately handled and prevent unintended or duplicate task executions.

<figure class="image">
  <img src="https://github.com/gumberss/Schedulinator/assets/38296002/7859d3d8-e625-414b-a800-4eb36e85ba04" alt="Resend the request with the task score and ID as the idempotency key">
  <figcaption>Resend the request with the task score and ID as the idempotency key</figcaption>
</figure>
 <br/><br/>
 
### Retry Policy

While the retry policy functions well for one-time tasks, it's important to ensure that the recurrence interval for repeated tasks is set longer than the worst-case scenario for the retry policy. This prevents a cascading effect of retries and recurring events. You can determine the retry policy for recurring tasks using the following formula:

![image](https://github.com/gumberss/Schedulinator/assets/38296002/e77a21a1-7b36-4a63-a6ef-a5471fca7e4d)

As an example, let's consider the retry policy with the following parameters: attempts=3, interval=200ms, and jitterLimit=500ms. This policy implies that the system will attempt a maximum of 3 retries. Each retry will occur after a time interval calculated as 200ms multiplied by 2 raised to the power of the current attempt, plus a random value from 0 to the jitterLimit value. Therefore, in the worst case of this example, as will be illustrated below, the minimum recurrence time for the tasks should be set to 2900ms or 2.9 seconds.

![image](https://github.com/gumberss/Schedulinator/assets/38296002/723b59d7-0dd6-4286-a301-ac8fb9f05726)

Step by step:

Attempt 1: (200ms * 2^0) + jitter (up to 500ms). Worst case: 200 + 500 = 700ms

Attempt 2: (200ms * 2^1) + jitter (up to 500ms). Worst case: 400 + 500 = 900ms

Attempt 3: (200ms * 2^2) + jitter (up to 500ms). Worst case: 800 + 500 = 1300ms

The sum of all the attempts' worst cases will be 2900ms

### Multiple Instances

As we aim to scale to millions of schedules, the need arises for running multiple instances of the system concurrently. However, this isn't a straightforward endeavor, given that our primary source of tasks for execution relies on Sorted Sets, and we want to prevent task loss in the event of an instance failure. Let's explore some potential solutions:

#### Global Timestamp

We can establish a global timestamp in Redis and each instance would set the current time to this global variable, Subsequently, it would retrieve the tasks scheduled for execution from the previous global timestamp value up to the present moment.

Drawbacks:

1. When we have 100 instances of this service running concurrently, it's possible that a significant number of them may not process any tasks. This can occur because many tasks may share the same millisecond for execution. For instance, when using cron expressions, it's common to schedule tasks to run every hour, minute, and second, typically starting at the first millisecond of that second. Consequently, instances that execute during the initial millisecond of execution will process a higher number of jobs, while instances that execute during other milliseconds will handle fewer tasks.

2. It's challenging to guarantee that all instances have precisely synchronized time. Even a minor time difference, such as one instance being 10 milliseconds ahead of the others, can lead to that instance effectively blocking the execution of the others for at least 10 milliseconds.

#### Binary Locking 

The default locking mechanism offers a potential solution. We can create a key with an associated lock for each item in the Sorted Set that is taken for processing and then release the lock when the task is completed. 

Drawbacks:

1. Identifying which tasks are currently in progress within the sorted set can be a complex task. In the proposed approach, each instance of the system would need to fetch the top X tasks and then check if they are locked. If a task is locked, the instance would have to skip those X tasks and retry until it finds unlocked items. However, with 100 instances of the service, this could potentially result in skipping a significant number of tasks (99 * X) before successfully obtaining a batch of X unlocked items. This approach may not be efficient for large-scale systems.

2. If an instance goes down after locking tasks but before releasing them, we face the challenge of identifying and resolving stuck tasks. This requires scanning the Redis keys to locate tasks that remain locked. To determine if a task is genuinely stuck and not merely in the processing phase, we must retrieve the task's score. A similar issue arises if we consider the approach of adding locked tasks to a set and removing them when processing is completed. We want to avoid O(N) operations.

#### Negative Elements Locking

In this solution, when an instance takes tasks for processing, it multiplies the score of these tasks by -1. Consequently, tasks with scores less than 0 are considered locked by an instance of the service and should not be processed by other instances.

To achieve this, the instances must execute an atomic Lua script to acquire the first X tasks and update their scores by multiplying them by -1. This ensures that no other instance will attempt to process the same tasks simultaneously. After completing the processing of tasks, the instance will update the scores to reflect the next execution time. Alternatively, instead of setting a static next recurrence time, we can calculate it by adding a specified interval to the last processing time. This is possible because we have access to the last task score by simply multiplying it by -1 again so determining the new execution time becomes straightforward using the score adding the recurrency time from the task configuration.

To implement this, we can create a global key with a timestamp that represents the next scheduled check for stuck tasks. Whenever an instance completes its processing, it checks if this timestamp has expired. If it has, the instance updates the timestamp to the next scheduled check time and proceeds to identify and handle any stuck tasks. If, for any reason, the instance goes down after updating the timestamp but before checking the tasks, another instance will perform the check once the scheduled time has expired again.

### Limitation

Every system has the potential to become an infinite game, and this complexity becomes more pronounced when addressing the challenges of distributed systems. As discussed earlier, this project serves as a proof of concept to demonstrate the effective scheduling of a significant volume of recurring tasks, with scalability being the primary goal. While additional features could enhance the system's functionality, they are considered beyond the POC's scope. Below, we will briefly mention some of these potential features for reference.

#### Tracking Task Executions

Depending on your project's requirements, you may need to monitor how many times a scheduled task has been executed. Although this may seem straightforward, it can present challenges. Various issues can arise during the process, making it complex to maintain an accurate count of task executions.

One simple solution for tracking the number of successful task executions involves incrementing a database record for each task. Alternatively, you can create a compound unique index in the database, combining the task ID and the expected execution time. These approaches provide a count of 'at least' how many times a task has been successfully executed.

#### Scheduling Events in the Message Queue

This project serves as a proof of concept, where making a request and publishing an event on the message queue are conceptually similar from a business logic perspective. Many of the challenges previously discussed can also occur with messages, but idempotency can effectively address these issues. Currently, our focus is on sending requests."

### Limitation

Every system has the potential to become an infinite game, and this complexity becomes more pronounced when addressing the challenges of distributed systems, as discussed earlier. This project serves as a proof of concept to demonstrate the effective scheduling of a significant volume of recurring tasks, with scalability being the primary goal. While additional features could enhance the system's functionality, they are considered beyond the POC's scope. Below, we will briefly mention some of these potential features for reference.

#### Execution Count

Depending on your project's business rules, you may need to keep track of the number of times a scheduled task has been successfully executed. While this may seem like a straightforward task, it can introduce complexities. Several challenges can emerge during this process, making it difficult to ensure an accurate count of task executions.

One relatively simple solution for tracking the number of successful task executions is to increment a database record for each task or create a compound unique index in the database based on the task ID and the expected execution time. These approaches provide a count of how many times 'at least' a task has been executed successfully.

#### Scheduling Events in a Message Queue

This project serves as a proof of concept, where making a request and publishing an event on the message queue are conceptually equivalent in terms of the service's business logic. Many of the issues mentioned earlier can also arise when dealing with messages, and idempotency mechanisms can help mitigate them. For the time being, we will limit our discussion to sending requests.

#### Redis Unreachable 

Another potential limitation is the case when Redis becomes unreachable. While there are various strategies to handle such situations, we will explore some of them shortly. These strategies become essential when you require high availability, mainly in scenarios where Redis replication spans multiple regions, and all of them become unreachable. It's worth noting that this is a highly specific use case and won't be part of our Proof of Concept (POC) at this time.

##### Persistent DB Synced with Redis

One solution would involve updating not only the sorted set with the new score each time the task runs but also the persistent database. In this configuration, even if Redis becomes unreachable, the system can continue to operate effectively, ensuring availability.

###### Drawbacks

Synchronizing Redis with the persistent database in every execution can significantly increase the cost of updating data in the database. In this case, Redis  would primarily helps in reducing the need to read data from the database, not the write. Therefore, I recommend carefully evaluating whether this approach is a cost-effective solution for your specific use case.

##### Critical Operation Only

Another solution is to assign a criticality value to tasks and operate only on critical tasks until Redis becomes available again. This approach helps conserve database resources by avoiding unnecessary reads and writes for non-critical tasks.

###### Drawbacks

Tasks with lower criticality will remain dormant until Redis becomes reachable again.

#### Re-synchronize Redis

When Redis goes down in all replicas and comes back up, it won't have any data stored in memory. To restore the system's operation, we need to re-add all the schedules to memory. We will explore some of the possibilities for doing this below.

##### Pessismitic Locking

One approach could involve each instance of the service selecting certain tasks from the database and locking them until they are updated in Redis using pessimistic locking ([example in postgress](https://www.2ndquadrant.com/en/blog/what-is-select-skip-locked-for-in-postgresql-9-5/)).

##### Optimistic locking

If you're not familiar with how Optimistic Locking works, you can refer to [this article](https://www.2ndquadrant.com/en/blog/postgresql-anti-patterns-read-modify-write-cycles/) can explain it for you.

In the solution described below, you can consider the 'last-sync-id' as the version key, and 'sync-owner' as a way to ensure that the instance responsible for the synchronization process was the one that acquired the lock.

The database can have two columns for synchronization: 'last-sync-id' and 'sync-owner.' When the first instance detects that Redis is back online, it can insert a random synchronization ID, referred to as 'current-sync-id,' into Redis under a specific key. Each instance can then retrieve this 'current-sync-id,' identify the top X elements in the database with a 'last-sync-id' different from the 'current-sync-id,' and update their 'last-sync-id' with the 'current-sync-id' and 'sync-owner' with an ID generated by that instance.

Subsequently, this instance of the service can select the top X elements that have 'last-sync-id' matching the 'current-sync-id' and 'sync-owner' set to the ID generated by itself. 

This solution enables the system to prioritize the restoration of critical tasks. Once all critical tasks have been restored, lower-criticality tasks can follow suit. The effectiveness of this approach depends on the database's design and whether a 'criticality' column has been defined.

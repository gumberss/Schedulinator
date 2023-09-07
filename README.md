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

One possible approach to mitigate the other service timeout issue is to always send a message or request to the endpoint with a unique identifier generated deterministically based on the score of the task within the Sorted Set. By doing so, you can rely on the receiving service to leverage idempotency to handle the problem. This way, even if the data within Redis becomes temporarily out of date due to a server failure, the unique identifier associated with each task can help ensure that duplicate requests are appropriately handled and prevent unintended or duplicate task executions.

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










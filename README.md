# Schedulinator

The core objective of this project is to develop a proof of concept for a high-performance scheduler capable of efficiently processing millions of tasks per second, leveraging the power of Redis Sorted Sets. This initiative seeks to identify the next tasks directly through Redis, thus eliminating the need for expensive database reads. Before using this project, it's crucial to conduct a thorough analysis to determine if it aligns with your specific needs. For instance, if your execution rate is relatively low, opting for Redis may not be the most cost-effective choice. Redis incurs costs based on the number of nodes (when using AWS at least), so in such cases, using a database like DynamoDb directly could be a more economical alternative.

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




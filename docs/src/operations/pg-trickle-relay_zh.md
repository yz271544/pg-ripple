# pg-trickle 中继已迁移

pg-trickle 中继不再存在。中继、outbox、inbox、消费者组和中继进程功能现在位于 pg_tide 中。

使用当前指南：[pg-tide 中继：中心辐射](pg-tide-relay.md)。

pg_trickle 仍由 pg_ripple 用于增量视图维护（SPARQL 视图、Datalog 视图、CONSTRUCT/DESCRIBE/ASK 视图和 ExtVP），但不用于中继传输。

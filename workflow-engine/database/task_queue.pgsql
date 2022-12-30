create table workflow_engine.task_queue (
    workflow_run_id bigint not null references workflow_engine.workflow_runs match simple
        on delete restrict
        on update cascade,
    task_order int not null,
    task_id bigint not null references workflow_engine.tasks match simple
        on delete restrict
        on update cascade,
    status workflow_engine.task_status not null default 'Waiting'::workflow_engine.task_status,
    parameters jsonb,
    output text check(workflow_engine.check_not_blank_or_empty(output)),
    rules workflow_engine.task_rule[] check(workflow_engine.are_valid_task_rules(rules)),
    task_start timestamp without time zone,
    task_end timestamp without time zone,
    constraint task_queue_pk primary key (workflow_run_id, task_order)
) partition by list(workflow_run_id);

revoke all on workflow_engine.task_queue from public;

comment on table workflow_engine.task_queue is 'Single tasks for a given workflow run. Partitioned by workflow run';
comment on column workflow_engine.task_queue.workflow_run_id is 'Id of the parent workflow run of this record';
comment on column workflow_engine.task_queue.task_order is 'Sequential order of task within the workflow run';
comment on column workflow_engine.task_queue.task_id is 'Id of the task to be executed';
comment on column workflow_engine.task_queue.status is 'Current status of the task';
comment on column workflow_engine.task_queue.parameters is 'Parameters passed to the task as unstructured data for custom actions';
comment on column workflow_engine.task_queue.output is 'Message output as result of task. Usually empty and filled when error occurs';
comment on column workflow_engine.task_queue.rules is 'Collection of all rules checked/run during task. Failed rules will halt workflow run';
comment on column workflow_engine.task_queue.task_start is 'Timestamp when task starts';
comment on column workflow_engine.task_queue.task_end is 'Timestamp of when task ends. Only populated when done but status values other than ''Complete'' can have a value here';
comment on constraint workflow_engine.task_queue.task_queue_pk is 'Records in task queue are unique for a task order per workflow run';

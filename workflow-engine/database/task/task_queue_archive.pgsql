create table if not exists task.task_queue_archive (
    workflow_run_id bigint not null,
    task_order int not null,
    task_id bigint not null,
    status task.task_status not null default 'Waiting'::task.task_status,
    parameters jsonb,
    output text check(data_check.check_not_blank_or_empty(output)),
    rules task.task_rule[] check(rules != '{}'::task.task_rule[]),
    task_start timestamp without time zone,
    task_end timestamp without time zone
);

create index if not exists wr_id
on task.task_queue_archive(workflow_run_id);
create index if not exists wr_id_task_ord
on task.task_queue_archive(workflow_run_id,task_order);
create index if not exists wr_id_task_id
on task.task_queue_archive(workflow_run_id,task_id);

comment on table task.task_queue_archive is
'Archive of entires from task_queue that were deleted or altered due to user requests';
comment on column task.task_queue_archive.workflow_run_id is
'Id of the parent workflow run of this record';
comment on column task.task_queue_archive.task_order is
'Sequential order of task within the workflow run';
comment on column task.task_queue_archive.task_id is
'Id of the task to be executed';
comment on column task.task_queue_archive.status is
'Current status of the task';
comment on column task.task_queue_archive.parameters is
'Parameters passed to the task as unstructured data for custom actions';
comment on column task.task_queue_archive.output is
'Message output as result of task. Usually empty and filled when error occurs';
comment on column task.task_queue_archive.rules is
'Collection of all rules checked/run during task. Failed rules will halt workflow run';
comment on column task.task_queue_archive.task_start is
'Timestamp when task starts';
comment on column task.task_queue_archive.task_end is $$
Timestamp of when task ends. Only populated when done but status values other than ''Complete''
can have a value here
$$;

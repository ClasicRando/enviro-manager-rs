create or replace function workflow_run.task_queue_record_archive()
returns trigger
language plpgsql
as $$
begin
    insert into workflow_run.task_queue_archive(
        workflow_run_id,task_order,task_id,status,parameters,output,rules,task_start,task_end
    )
    select
        tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, tq.output,
        tq.rules, tq.task_start, tq.task_end
    from old_table tq;
    return null;
end;
$$;

create table if not exists workflow_run.task_queue (
    workflow_run_id bigint not null references workflow_run.workflow_runs match simple
        on delete restrict
        on update cascade,
    task_order int not null,
    task_id bigint not null references workflow.tasks match simple
        on delete restrict
        on update cascade,
    status workflow_run.task_status not null default 'Waiting'::workflow_run.task_status,
    parameters jsonb,
    output text check(data_check.check_not_blank_or_empty(output)),
    rules workflow_run.task_rule[],
    task_start timestamp without time zone,
    task_end timestamp without time zone,
    progress smallint check(case when progress is not null then progress between 0 and 100 else true end),
    constraint task_queue_pk primary key (workflow_run_id, task_order)
) partition by list(workflow_run_id);

create or replace trigger record_update
    after update
    on workflow_run.task_queue
    referencing old table as old_table
    for each statement
    execute function workflow_run.task_queue_record_archive();

create or replace trigger record_update
    after delete
    on workflow_run.task_queue
    referencing old table as old_table
    for each statement
    execute function workflow_run.task_queue_record_archive();

comment on table workflow_run.task_queue is
'Single tasks for a given workflow run. Partitioned by workflow run';
comment on column workflow_run.task_queue.workflow_run_id is
'Id of the parent workflow run of this record';
comment on column workflow_run.task_queue.task_order is
'Sequential order of task within the workflow run';
comment on column workflow_run.task_queue.task_id is
'Id of the task to be executed';
comment on column workflow_run.task_queue.status is
'Current status of the task';
comment on column workflow_run.task_queue.parameters is
'Parameters passed to the task as unstructured data for custom actions';
comment on column workflow_run.task_queue.output is
'Message output as result of workflow_run. Usually empty and filled when error occurs';
comment on column workflow_run.task_queue.rules is
'Collection of all rules checked/run during workflow_run. Failed rules will halt workflow run';
comment on column workflow_run.task_queue.task_start is
'Timestamp when task starts';
comment on column workflow_run.task_queue.task_end is $$
Timestamp of when task ends. Only populated when done but status values other than ''Complete''
can have a value here
$$;
comment on column workflow_run.task_queue.progress is
'Progress toward task completion. If not null then between 0 and 100';
comment on constraint task_queue_pk on workflow_run.task_queue is
'Records in task queue are unique for a task order per workflow run';

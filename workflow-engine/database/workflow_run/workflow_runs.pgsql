create or replace function workflow_run.workflow_run_status_event()
returns trigger
language plpgsql
as $$
declare
    v_next_executor bigint;
    v_job_id bigint;
begin
    if new.status = 'Scheduled'::workflow_run.workflow_run_status and new.executor_id is null then
        v_next_executor := executor.next_executor();
        if v_next_executor is not null then
            new.executor_id = v_next_executor;
            perform pg_notify('wr_scheduled_'||v_next_executor, new.workflow_run_id::text);
        end if;
    elsif new.status = 'Canceled'::workflow_run.workflow_run_status and old.executor_id is not null then
        perform pg_notify('wr_canceled_'||old.executor_id, new.workflow_run_id::text);
    end if;

    select j.job_id
    into v_job_id
    from job.jobs j
    where j.current_workflow_run_id = new.workflow_run_id;

    if v_job_id is not null and new.status not in (
        'Scheduled'::workflow_run.workflow_run_status,
        'Running'::workflow_run.workflow_run_status
    ) then
        perform pg_notify('jobs', v_job_id::text);
    end if;
    return new;
end;
$$;

create or replace function workflow_run.workflow_progress_event()
returns trigger
language plpgsql
as $$
begin
    if new.progress is not null and new.progress != coalesce(old.progress,0) then
        perform pg_notify('wr_progress', new.workflow_run_id::text);
    end if;
    return new;
end;
$$;

create table if not exists workflow_run.workflow_runs (
    workflow_run_id bigint primary key generated always as identity,
    workflow_id bigint not null references workflow.workflows match simple
        on delete restrict
        on update cascade,
    status workflow_run.workflow_run_status not null default 'Waiting'::workflow_run.workflow_run_status,
    executor_id bigint references executor.executors match simple
        on delete set null
        on update cascade,
    progress smallint check(case when progress is not null then progress between 0 and 100 else true end)
);

create or replace trigger workflow_run_status
    before update of status
    on workflow_run.workflow_runs
    for each row
    execute function workflow_run.workflow_run_status_event();

create or replace trigger workflow_run_progress
    before update of progress
    on workflow_run.workflow_runs
    for each row
    execute function workflow_run.workflow_progress_event();

call audit.audit_table('workflow_run.workflow_runs');

comment on table workflow_run.workflow_runs is
'Run instance of a given generic workflow';
comment on column workflow_run.workflow_runs.workflow_run_id is
'Unique identifier for each workflow run';
comment on column workflow_run.workflow_runs.workflow_id is
'Id of the templated workflow executed during the run';
comment on column workflow_run.workflow_runs.status is
'Current status of the workflow run';
comment on column workflow_run.workflow_runs.executor_id is
'Id of the executor that owns this workflow run. Is null until picked up by executor';
comment on column workflow_run.workflow_runs.progress is
'Optional progress that the worker reports as iterations/subtasks are completed';
comment on trigger workflow_run_status on workflow_run.workflow_runs is
'Trigger run during status updates to notify the required listeners of changes';
comment on trigger workflow_run_progress on workflow_run.workflow_runs is
'Trigger run during progress updates to notify the required listeners of changes';

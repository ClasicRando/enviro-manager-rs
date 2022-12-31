create or replace function workflow_engine.job_change()
returns trigger
language plpgsql
as $$
begin
    perform pg_notify('jobs', '');
    return null;
end;
$$;

create table if not exists workflow_engine.jobs (
    job_id bigint primary key generated always as identity,
    workflow_id bigint not null references workflow_engine.workflows match simple
        on delete restrict
        on update cascade,
    job_type workflow_engine.job_type not null,
    maintainer text not null check(data_check.check_not_blank_or_empty(maintainer)),
    job_interval interval check(
        case
            when job_type = 'Interval'::workflow_engine.job_type
                then job_interval is not null and job_interval > interval '0 second'
            else job_interval is null
        end
    ),
    job_schedule workflow_engine.schedule_entry[] check(
        case
            when job_type = 'Scheduled'::workflow_engine.job_type
                then workflow_engine.valid_job_schedule(job_schedule)
            else job_schedule is null
        end
    ),
    is_paused boolean not null default false,
    next_run timestamp without time zone not null check(next_run > now() at time zone 'UTC'),
    current_workflow_run_id bigint references workflow_engine.workflow_runs match simple
        on delete restrict
        on update cascade
);

drop trigger if exists job_change_trig on workflow_engine.jobs;
create trigger job_change_trig
    after update or insert or delete
    on workflow_engine.jobs
    for each statement
    execute function workflow_engine.job_change();

call audit.audit_table('workflow_engine.jobs');

revoke all on workflow_engine.jobs from public;

comment on table workflow_engine.jobs is 'Jobs to be run periodically as definied by the jobs''s schedule/interval';
comment on column workflow_engine.jobs.job_id is 'Unique identifier for each job';
comment on column workflow_engine.jobs.workflow_id is 'Id of the templated workflow executed during the job run';
comment on column workflow_engine.jobs.job_type is 'Variant of job. If interval, job_interval is non-null. If scheduled, job_schedule is non-null';
comment on column workflow_engine.jobs.maintainer is 'Email address to send error notifications if the job failed to run, or a runtime error occured';
comment on column workflow_engine.jobs.job_interval is 'Interval defining when the next run should occur. Relative to the last run datetime. Keep in mind runtime when choosing interval for frequent jobs';
comment on column workflow_engine.jobs.job_schedule is 'Schedule within a week as to when the job should be run. Allows for uneven running but is restricted to at least a weekly run';
comment on column workflow_engine.jobs.is_paused is 'Indicates a user flagged this job to be paused or the most recent job failed and the job is automatically set to paused to avoid re-run issues';
comment on column workflow_engine.jobs.next_run is 'Next time the job should be run. Decided by the schedule/interval';
comment on column workflow_engine.jobs.current_workflow_run_id is 'If the job is currently running, this will link to a workflow_run record';
comment on trigger job_change_trig on workflow_engine.jobs is 'Trigger run during any change to the records to notify the job worker of new changes.';

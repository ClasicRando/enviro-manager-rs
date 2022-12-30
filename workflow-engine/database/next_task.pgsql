create or replace function workflow_engine.next_task(
    workflow_run_id bigint
) returns table (
    workflow_run_id bigint,
    task_order integer,
    task_id bigint,
    parameters jsonb,
    url text
)
language sql
stable
as $$
select tq1.workflow_run_id, tq1.task_order, tq1.task_id, tq1.parameters,
       rtrim(ts.base_url,'/') || '/' || ltrim(t.url,'/') url
from   workflow_engine.task_queue tq1
join   workflow_engine.tasks t on tq1.task_id = t.task_id
join   workflow_engine.task_services ts on ts.service_id = t.task_service_id
where  tq1.workflow_run_id = $1
and    not exists(
    select 1
    from   workflow_engine.task_queue tq2
    where  tq1.workflow_run_id = tq2.workflow_run_id
    and    tq2.status in (
        'Running'::workflow_engine.task_status,
        'Paused'::workflow_engine.task_status,
        'Failed'::workflow_engine.task_status,
        'Rule Broken'::workflow_engine.task_status
    )
)
and    tq1.status = 'Waiting'::workflow_engine.task_status
order by tq1.task_order
limit 1
for update;
$$;

comment on function workflow_engine.next_task IS $$
Get the next available task for the given workflow_run_id. Returns at most 1 row of a row
containing data about the executable task.

!NOTE! This function locks the record so this should be run within a transaction and once the
record is updated, immediately commit or rollback on error.

Arguments:
workflow_run_id:    ID of the workflow run to check for the next task
$$;

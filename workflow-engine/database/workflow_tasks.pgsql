create function workflow_engine.workflow_tasks_check()
returns trigger
language plpgsql
stable
as $$
declare
    v_workflow_id bigint;
    v_errors text;
begin
    if TG_OP in ('UPDATE', 'INSERT') then
        begin
            select distinct workflow_id 
            into   v_workflow_id
            from   new_table
        exception
            when too_many_rows then
                raise exception 'When affecting workflow_tasks, only 1 workflow_id can be impacted';
        end;
    else
        begin
            select distinct workflow_id 
            into   v_workflow_id
            from   old_table
        exception
            when too_many_rows then
                raise exception 'When affecting workflow_tasks, only 1 workflow_id can be impacted';
        end;
    end if;

    select string_agg(format('task_id = %s, expected %s, got %s', task_id, rn, task_order), chr(10))
    into   v_errors
    from (
        select task_id, task_order, row_number() over (order by task_order) rn
        from   workflow_engine.workflow_tasks
        where  workflow_id = v_workflow_id
    )
    where  task_order != rn;

    if v_errors is not null then
        raise exception 'Task order values are not correct. See these instances:'||chr(10)||'%s', v_errors;
    end if;

    return null;
end;
$$;

create table workflow_engine.workflow_tasks (
    workflow_id bigint not null references workflow_engine.workflows match simple
        on delete restrict
        on update cascade,
    task_order integer not null,
    task_id bigint not null references workflow_engine.tasks match simple
        on delete restrict
        on update cascade,
    parameters jsonb,
    constraint workflow_tasks_pk primary key(workflow_id, task_order)
);

create trigger verify_records
    after insert or update or delete
    on workflow_engine.workflow_tasks
    referencing new table as new_table old table as old_table
    for each statement
    execute function workflow_engine.workflow_tasks_check();

call audit.audit_table('workflow_engine.workflow_tasks');

revoke all on workflow_engine.workflow_tasks from public;

comment on table workflow_engine.workflow_tasks is 'All tasks linked to a parent workflow';
comment on column workflow_engine.workflow_tasks.workflow_id is 'Id of the parent workflow';
comment on column workflow_engine.workflow_tasks.task_order is 'Order within the workflow. Must be sequential with no gaps';
comment on column workflow_engine.workflow_tasks.task_id is 'Id of the task to be executed in the parent workflow at this order position';
comment on column workflow_engine.workflow_tasks.parameters is 'Parameters to be passed to the executing service to customize behaviour';
comment on trigger workflow_engine.workflow_tasks.verify_records is 'Trigger to guarentee that a single workflow_id is impacted and the task_order values the sequential with no gaps';

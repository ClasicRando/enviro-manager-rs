create or replace function workflow.workflow_tasks_check()
returns trigger
language plpgsql
volatile
as $$
declare
    v_workflow_id bigint;
    v_workflow_id_count int;
    v_errors text;
begin
    if TG_OP in ('UPDATE', 'INSERT') then
        select count(distinct nt.workflow_id)
        into v_workflow_id_count
        from new_table nt;
        
        select nt.workflow_id
        into v_workflow_id
        from new_table nt
        limit 1;
    else
        select count(distinct ot.workflow_id)
        into v_workflow_id_count
        from old_table ot;
        
        select ot.workflow_id
        into v_workflow_id
        from old_table ot
        limit 1;
    end if;
    
    if v_workflow_id_count > 1 then
        raise exception 'When affecting workflow_tasks, only 1 workflow_id can be impacted';
    end if;

    select
        string_agg(
            format(
                'task_id = %s, expected %s, got %s',
                e.task_id,
                e.rn,
                e.task_order
            ),
            chr(10)
        )
    into v_errors
    from (
        select wt.task_id, wt.task_order, row_number() over (order by wt.task_order) rn
        from workflow.workflow_tasks wt
        where wt.workflow_id = v_workflow_id
    ) e
    where e.task_order != rn;

    if v_errors is not null then
        raise exception 'Task order values are not correct. See these instances: %', v_errors;
    end if;

    return null;
end;
$$;

create table if not exists workflow.workflow_tasks (
    workflow_id bigint not null references workflow.workflows match simple
        on delete restrict
        on update cascade,
    task_order integer not null,
    task_id bigint not null references workflow.tasks match simple
        on delete restrict
        on update cascade,
    parameters jsonb,
    constraint workflow_tasks_pk primary key(workflow_id, task_order)
);

create or replace trigger verify_insert_records
    after insert
    on workflow.workflow_tasks
    referencing new table as new_table
    for each statement
    execute function workflow.workflow_tasks_check();

create or replace trigger verify_update_records
    after update
    on workflow.workflow_tasks
    referencing new table as new_table old table as old_table
    for each statement
    execute function workflow.workflow_tasks_check();

create or replace trigger verify_delete_records
    after delete
    on workflow.workflow_tasks
    referencing old table as old_table
    for each statement
    execute function workflow.workflow_tasks_check();

call audit.audit_table('workflow.workflow_tasks');

comment on table workflow.workflow_tasks is
'All tasks linked to a parent workflow';
comment on column workflow.workflow_tasks.workflow_id is
'Id of the parent workflow';
comment on column workflow.workflow_tasks.task_order is
'Order within the workflow. Must be sequential with no gaps';
comment on column workflow.workflow_tasks.task_id is
'Id of the task to be executed in the parent workflow at this order position';
comment on column workflow.workflow_tasks.parameters is
'Parameters to be passed to the executing service to customize behaviour';
comment on trigger verify_insert_records on workflow.workflow_tasks is $$
Trigger to guarantee that a single workflow_id is inserted and the task_order values the
sequential with no gaps
$$;
comment on trigger verify_update_records on workflow.workflow_tasks is $$
Trigger to guarantee that a single workflow_id is inserted and the task_order values the
sequential with no gaps
$$;
comment on trigger verify_delete_records on workflow.workflow_tasks is $$
Trigger to guarantee that a single workflow_id is inserted and the task_order values the
sequential with no gaps
$$;

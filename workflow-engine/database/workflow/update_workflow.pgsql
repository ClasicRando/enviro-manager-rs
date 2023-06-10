create or replace procedure workflow.update_workflow(
    workflow_id bigint,
    name text
)
security definer
language sql
as $$
update workflow.workflows w
set name = $2
where w.workflow_id = $1
$$;

grant execute on procedure workflow.update_workflow to we_web;

comment on procedure workflow.update_workflow IS $$
Update the existing workflow to the new name.

Arguments:
workflow_id:
    ID of the workflow to update
name:
    New alias given to the new workflow
$$;

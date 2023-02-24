create or replace procedure workflow_engine.deprecate_workflow(
    workflow_id bigint,
    new_workflow_id bigint default null
)
language sql
as $$
update workflow_engine.workflows w
set
    is_deprecated = true,
    new_workflow = $2
where w.workflow_id = $1;
$$;

comment on procedure workflow_engine.deprecate_workflow IS $$
Set workflow as deprecated and optional point to the new workflow to be used

Arguments:
workflow_id:
    ID of the workflow to be deprecated
new_workflow_id:   
    Optional parameter as the replacement workflow
$$;

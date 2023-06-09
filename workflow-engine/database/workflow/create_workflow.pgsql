create or replace function workflow.create_workflow(
    name text
) returns bigint
security definer
language sql
as $$
insert into workflow.workflows(name)
values($1)
returning workflow_id
$$;

grant execute on function workflow.create_workflow to we_web;

comment on function workflow.create_workflow IS $$
Create a new template workflow, aliased as the provided name. Returns the new workflow id.

Arguments:
name:
    Alias given to the new workflow
$$;

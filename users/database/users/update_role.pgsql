create or replace function users.update_role(
    in_name text,
    in_new_name text,
    in_new_description text,
    out name text,
    out description text
)
returns record
volatile
language sql
as $$
update users.roles r
set
    name = coalesce(nullif(trim($2),''), r.name),
    description = coalesce(nullif(trim($3),''), r.description)
where r.name = $1
returning r.name, r.description
$$;

grant execute on function users.update_role to users_web;

comment on function users.update_role IS $$
Update the name and/or the description of a role specified by the name parameter. If either new
value is null then the original value is kept.

Arguments:
action_uid:
    User ID that is attempting to perform the action
name:
    Name of the existing role to update
new_name:
    New name to update the existing role. Will not be updated is the input value is null. If a new
    value is provided, it must be unique within the roles table
new_description:
    New long description of what actions a role allows a user to perform. Will not be updated is
    the input value is null.
$$;

create or replace procedure users.update_role(
    action_em_uid bigint,
    name text,
    new_name text default null,
    new_description text default null,
    out r_name text,
    out r_description text
)
language plpgsql
as $$
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role($1, 'create role');
    update users.roles r
    set
        name = coalesce($3, r.name),
        description = coalesce($4, r.description)
    where r.name = $2
    returning r.name, r.description into $5, $6;
end;
$$;

grant execute on procedure users.update_role to users_web;

comment on procedure users.update_role IS $$
Update the name and/or the description of a role specified by the name parameter. If either new
value is null then the original value is kept.

Arguments:
action_em_uid:
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

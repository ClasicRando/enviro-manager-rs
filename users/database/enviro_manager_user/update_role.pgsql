create or replace procedure enviro_manager_user.update_role(
    name text,
    new_name text default null,
    new_description text default null
)
language plpgsql
as $$
declare
    v_uid bigint := enviro_manager_user.get_current_em_uid();
begin
    call enviro_manager_user.check_user_role(v_uid, 'create role');
    update enviro_manager_user.roles r
    set
        name = case when $3 is null then r.name else $3 end,
        description = case when $2 is null then r.description else $2 end
    where r.name = $1;
end;
$$;

comment on procedure enviro_manager_user.update_role IS $$
Update the name and/or the description of a role specified by the name parameter. If either new
value is null then the original value is kept.

Arguments:
name:
    Name of the existing role to update
new_name:
    New name to update the existing role. Will not be updated is the input value is null. If a new
    value is provided, it must be unique within the roles table
new_description:
    New long description of what actions a role allows a user to perform. Will not be updated is
    the input value is null.
$$;

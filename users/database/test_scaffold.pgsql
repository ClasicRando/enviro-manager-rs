declare
    v_admin_role users.roles := row(
        'admin',
        'Role with full access to all other roles'
    )::users.roles;
    v_create_user_role users.roles := row(
        'create-user',
        'Provides a user with the ability to create other users'
    )::users.roles;
    v_create_role_role users.roles := row(
        'create-role',
        'Provides a user with the ability to create/modify roles'
    )::users.roles;
    v_add_role_role users.roles := row(
        'add-role',
        'Provides a user with the ability to add/remove roles from a user'
    )::users.roles;
    v_update_role_test1 users.roles := row(
        'update-role-1',
        'Test role to update'
    )::users.roles;
    v_update_role_test2 users.roles := row(
        'update-role-2',
        'Test role to update'
    )::users.roles;
    v_update_role_test4 users.roles := row(
        'update-role-3',
        'Test role to update'
    )::users.roles;
    v_update_role_test3 users.roles := row(
        'update-role-4',
        'Test role to update'
    )::users.roles;
    v_roles users.roles[] := array[
        v_admin_role,
        v_create_user_role,
        v_create_role_role,
        v_add_role_role,
        v_update_role_test1,
        v_update_role_test2,
        v_update_role_test3,
        v_update_role_test4
    ];
    v_em_uids bigint[];
begin
	truncate users.user_roles, users.roles, users.users;
    -- Create base roles
    insert into users.roles(name, description)
    select r.name, r.description
    from unnest(v_roles) r;

    -- Create test users
    with users as (
        insert into users.users(uid, first_name, last_name, username, password)
        values
            ('9363ab3f-0d62-4b40-b408-898bdea56282'::uuid, 'Admin', 'This is', 'admin', crypt('admin', gen_salt('bf'))),
            ('1cc58326-84aa-4c08-bb91-8c4536797e8c'::uuid, 'Create-User', 'This is', 'create-user', crypt('create-user', gen_salt('bf'))),
            ('bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca'::uuid, 'Create-Role', 'This is', 'create-role', crypt('create-role', gen_salt('bf'))),
            ('728ac060-9d38-47e9-b2fa-66d2954110e3'::uuid, 'Add-Role', 'This is', 'add-role', crypt('add-role', gen_salt('bf')))
        returning em_uid
    )
    select array_agg(em_uid)
    into v_em_uids
    from users;

    -- Set up test user roles
    insert into users.user_roles(em_uid, role)
    select a.em_uid, u.username
    from unnest(v_em_uids) a(em_uid)
    join users.users u
    on a.em_uid = u.em_uid;
end;
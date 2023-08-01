begin
	truncate users.user_roles, users.users restart identity;

    -- Create test users
    with new_users as (
        insert into users.users as u(uid, full_name, username, password)
        values
            ('9363ab3f-0d62-4b40-b408-898bdea56282'::uuid, 'Admin This is', 'admin', crypt('admin', gen_salt('bf'))),
            ('728ac060-9d38-47e9-b2fa-66d2954110e3'::uuid, 'Add-Role This is', 'add-role', crypt('add-role', gen_salt('bf'))),
            ('be4c1ef7-771a-4580-b0dd-ff137c64ab48'::uuid, 'None This is', 'none', crypt('none', gen_salt('bf')))
        returning u.uid, u.username
    )
    -- Set up test user roles
    insert into users.user_roles(uid, role)
    select u.uid, u.username
    from new_users u
    where u.username != 'none';
end;
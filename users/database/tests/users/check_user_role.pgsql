declare
    v_failed boolean;
    v_role text := 'create-user';
    v_admin_user uuid := '9363ab3f-0d62-4b40-b408-898bdea56282'::uuid;
    v_create_user_user uuid := '1cc58326-84aa-4c08-bb91-8c4536797e8c'::uuid;
    v_create_role_user uuid := 'bca9ff0f-06f8-40bb-9373-7ca0e10ed8ca'::uuid;
begin
    v_failed := false;
    begin
        call users.check_user_role(v_admin_user, v_role);
        v_failed := false;
    exception
        when others then
            v_failed := true;
    end;

    assert not v_failed, 'check_user_role should not fail when user is admin';

    v_failed := false;
    begin
        call users.check_user_role(v_create_user_user, v_role);
        v_failed := false;
    exception
        when others then
            v_failed := true;
    end;

    assert not v_failed, 'check_user_role should not fail when user has expected role';

    v_failed := false;
    begin
        call users.check_user_role(v_create_role_user, v_role);
        v_failed := false;
    exception
        when others then
            v_failed := true;
    end;

    assert v_failed, 'check_user_role should fail when user has expected role';
end;

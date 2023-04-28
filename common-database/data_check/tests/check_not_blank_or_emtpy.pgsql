do $$
declare
    v_check boolean;
    v_value text;
begin
    -- Input is null
    v_value := null;

    v_check := data_check.check_not_blank_or_empty(v_value);

    if not v_check then
        raise exception 'When input is null, should return true';
    end if;

    -- Input is empty
    v_value := '';

    v_check := data_check.check_not_blank_or_empty(v_value);

    if v_check then
        raise exception 'When input is empty, should return false';
    end if;

    -- Input is whitespace only
    v_value := '';

    v_check := data_check.check_not_blank_or_empty(v_value);

    if v_check then
        raise exception 'When input is whitespace only, should return false';
    end if;

    -- Input is valid text
    v_value := 'Test';

    v_check := data_check.check_not_blank_or_empty(v_value);

    if not v_check then
        raise exception 'When input is valid text, should return true';
    end if;
end;
$$;

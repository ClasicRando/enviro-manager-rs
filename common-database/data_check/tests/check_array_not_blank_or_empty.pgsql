do $$
declare
    v_check boolean;
    v_value text[];
begin
    -- Input is null
    v_value := null;

    v_check := data_check.check_array_not_blank_or_empty(v_value);

    if not v_check then
        raise exception 'When input is null, should return true';
    end if;

    -- Input is empty array
    v_value := array[]::text[];

    v_check := data_check.check_array_not_blank_or_empty(v_value);

    if v_check then
        raise exception 'When input is empty array, should return false';
    end if;

    -- Input is array with null entry
    v_value := array['Test', null]::text[];

    v_check := data_check.check_array_not_blank_or_empty(v_value);

    if v_check then
        raise exception 'When input is array with null entry, should return false';
    end if;

    -- Input is array with empty entry
    v_value := array['Test', '']::text[];

    v_check := data_check.check_array_not_blank_or_empty(v_value);

    if v_check then
        raise exception 'When input is array with empty entry, should return false';
    end if;

    -- Input is array with whitespace only entry
    v_value := array['Test', '   ']::text[];

    v_check := data_check.check_array_not_blank_or_empty(v_value);

    if v_check then
        raise exception 'When input is array with whitespace only entry, should return false';
    end if;

    -- Input is array with valid entries
    v_value := array['Test', 'Test2']::text[];

    v_check := data_check.check_array_not_blank_or_empty(v_value);

    if not v_check then
        raise exception 'When input is array with valid entries, should return true';
    end if;
end;
$$;

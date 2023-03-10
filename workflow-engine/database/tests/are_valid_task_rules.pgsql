declare
    v_value task.task_rule[];
    v_check boolean;
begin
    -- Input is null
    v_value := null;

    v_check := task.are_valid_task_rules(v_value);

    if not v_check then
        raise exception 'When input is null, should return true';
    end if;

    -- Input is empty array
    v_value := array[]::task.task_rule[];

    v_check := task.are_valid_task_rules(v_value);

    if v_check then
        raise exception 'When input is empty array, should return false';
    end if;

    -- Input is array with null name entry
    v_value := array[row(null,false,null)]::task.task_rule[];

    v_check := task.are_valid_task_rules(v_value);

    if v_check then
        raise exception 'When input is array with null name entry, should return false';
    end if;

    -- Input is array with empty name entry
    v_value := array[row('',false,null)]::task.task_rule[];

    v_check := task.are_valid_task_rules(v_value);

    if v_check then
        raise exception 'When input is array with null name entry, should return false';
    end if;

    -- Input is array with blank name entry
    v_value := array[row('   ',false,null)]::task.task_rule[];

    v_check := task.are_valid_task_rules(v_value);

    if v_check then
        raise exception 'When input is array with blank name entry, should return false';
    end if;

    -- Input is array with null failed flag entry
    v_value := array[row('Test',null,null)]::task.task_rule[];

    v_check := task.are_valid_task_rules(v_value);

    if v_check then
        raise exception 'When input is array with null failed flag entry, should return false';
    end if;

    -- Input is valid rules
    v_value := array[row('Test',false,null)]::task.task_rule[];

    v_check := task.are_valid_task_rules(v_value);

    if not v_check then
        raise exception 'When input is valid rules, should return true';
    end if;
end;

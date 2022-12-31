create or replace function workflow_engine.are_valid_task_rules(
    workflow_engine.task_rule[]
) returns boolean
language plpgsql
immutable
as $$
declare
    task_rule workflow_engine.task_rule;
begin
    if $1 is null then
        return true;
    end if;

    if $1 = '{}' then
        return false;
    end if;

    foreach task_rule in array $1
    loop
        if task_rule.name is null or 
            not data_check.check_not_blank_or_empty(task_rule.name) or
            task_rule.failed is null
        then
            return false;
        end if;
    end loop;
    return true;
end;
$$;

comment on function workflow_engine.are_valid_task_rules IS $$
Check to confirm array of task rules are valid. Returns false when the array is empty or any entry
contains a null/blank/empty name or a null failed flag.
$$;

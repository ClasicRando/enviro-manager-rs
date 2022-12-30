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
        if $1.name is null or 
            not workflow_engine.check_not_blank_or_empty($1.name) or
            $1.failed is null
        then
            return false;
        end if;
    end loop;
    return true;
end;
$$;

comment on type workflow_engine.task_rule IS $$
Check to confirm array of task rules are valid. Returns false when the array is empty or any entry
contains a null/blank/empty name or a null failed flag.
$$;

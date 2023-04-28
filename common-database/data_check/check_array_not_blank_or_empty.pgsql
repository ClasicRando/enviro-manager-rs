create or replace function data_check.check_array_not_blank_or_empty(
    text[]
) returns boolean
language plpgsql
immutable
as $$
declare
    val text;
begin
    if $1 is null then
        return true;
    end if;
    
    if $1 = '{}' then
        return false;
    end if;

    foreach val in array $1
    loop
        if val is null or val ~ '^\s*$' then
            return false;
        end if;
    END loop;

    return true;
end;
$$;

comment on function data_check.check_array_not_blank_or_empty IS $$
Check to confirm text array is not empty or contains values that are only whitespace or null.
Returns false when either criteria is met. Null values return true.
$$;

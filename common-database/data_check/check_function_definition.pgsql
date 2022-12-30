create or replace procedure data_check.check_function_definition(
    schema_name text,
    function_name text,
    expected_count integer
)
language plpgsql
as $$
declare
    v_count integer;
begin
    select count(0)
    into   v_count
    from   pg_proc p
    join   pg_namespace n on p.pronamespace = n.oid
    where  n.nspname = $1
    and    p.proname = $2;

    if v_count != $3 then
        raise exception 'Function %.% was expected to have % definition(s) but found %', $1, $2, $3, v_count;
    end if;
end;
$$;

comment on procedure data_check.check_function_definition IS $$
Checks pg catalog tables to ensure that a given schema qualified function only has the expected
number of definitions. Raises an exception if count does not match.

Arguments:
schema_name:    Schema name to check
function_name:  Function name to check
expected_count: Number of definitions that should be found
$$;

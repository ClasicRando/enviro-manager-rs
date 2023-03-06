create or replace procedure data_check.check_composite_definition(
    schema_name text,
    composite_name text,
    attributes text[]
)
language plpgsql
as $$
declare
    v_message text;
begin
    with db_attributes as (
        select
            a.attname,
            case
                when t2.typnamespace = t.typnamespace
                    then n.nspname||'.'||regexp_replace(t2.typname,'^_(.+)$','\1[]')
                else regexp_replace(t2.typname,'^_(.+)$','\1[]')
            end typname
        from pg_type t
        join pg_namespace n on t.typnamespace = n.oid
        join pg_class c on t.typrelid = c.oid
        join pg_attribute a on c.oid = a.attrelid
        join pg_type t2 on a.atttypid = t2.oid
        where
            n.nspname = $1
            and t.typname = $2
    ), code_attributes as (
        select
            regexp_substr(a, '[^ ]+', 1, 1) attname,
            case regexp_substr(a, '[^ ]+', 1, 2)
                when 'bigint' then 'int8'
                when 'int' then 'int4'
                when 'integer' then 'int4'
                when 'smallint' then 'int2'
                when 'boolean' then 'bool'
                else regexp_substr(a, '[^ ]+', 1, 2)
            end typname
        from unnest($3) a
    )
    select
        string_agg(
            case
                when da.typname is null then format('Database missing attribute "%s"', ca.typname)
                when ca.typname is null
                    then format('Extra attribute found in the database definition "%s"', da.typname)
                else format(
                    'Mismatch in attribute definition. Expected "%s", got "%s"',
                    da.typname,
                    ca.typname
                )
            end,
            ', '
        )
    into v_message
    from db_attributes da
    full join code_attributes ca on da.attname = ca.attname
    where
        da.typname is null
        or ca.typname is null
        or da.typname != ca.typname;
        
    if v_message is not null then
        raise exception '%', format('Found errors in %I.%I', $1, $2)||chr(10)||v_message;
    end if;
end;
$$;


comment on procedure data_check.check_composite_definition IS $$
Checks pg catalog tables to ensure that a given schema qualified composite matches the definition
found in the codebase.

Arguments:
schema_name:
    Schema name to check
enum_name:
    Function name to check
attributes:
    Collection of attributes (formatted as `name type`) expected for the composite
$$;

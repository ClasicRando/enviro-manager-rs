create or replace function data_check.trigger_exists(
    schema_name text,
    table_name text,
    type_name text
) returns boolean
returns null on null input
stable
language sql
as $$
select exists(
    select 1
    from pg_trigger trg
    join pg_class c on trg.tgrelid = c.oid
    join pg_namespace n on c.relnamespace = n.oid
    where
        n.nspname = $1
        and c.relname = $2
        and trg.tgname = $3
)
$$;

comment on function data_check.trigger_exists IS $$
Returns true if for the schema, table and trigger name given, there is an entry in the pg catalog
tables
$$;

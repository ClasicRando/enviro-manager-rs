create or replace function data_check.type_exists(
    schema_name text,
    type_name text
) returns boolean
returns null on null input
stable
language sql
as $$
select exists(
    select 1
    from pg_namespace n
    join pg_type t on n.oid = t.typnamespace
    where
        n.nspname = $1
        and t.typname = $2
)
$$;

comment on function data_check.type_exists IS $$
Returns true if for the schema and name given, there is an entry in the pg catalog tables
$$;

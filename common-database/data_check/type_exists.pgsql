create function data_check.type_exists(
    type_name text,
    schema_name text
) returns boolean
returns null on null input
stable
language sql
as $$
select exists(
    select 1
    from   pg_namespace n
    join   pg_type t on n.oid = t.typnamespace
    where  n.nspname = $2
    and    t.typname = $1
)
$$;

comment on function data_check.type_exists IS $$
Returns true if for the schema and name given, there is an entry in the pg catalog tables
$$;

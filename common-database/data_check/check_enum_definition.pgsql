create or replace procedure data_check.check_enum_definition(
    schema_name text,
    enum_name text,
    labels text[]
)
language plpgsql
as $$
declare
    v_message text;
begin
    with db_labels as (
        select e.enumlabel
        from   pg_enum e
        join   pg_type t on e.enumtypid = t.oid
        join   pg_namespace n on t.typnamespace = n.oid
        where  n.nspname = $1
        and    t.typname = $2
    ), code_labels as (
        select l enumlabel
        from   unnest($3) l
    )
    select string_agg(
            case
                when dl.enumlabel is null then format('Database missing label "%s"', cl.enumlabel)
                else format('Extra label "%s" found in database definition', dl.enumlabel)
            end,
            ', '
           )
    into   v_message
    from   db_labels dl
    full join code_labels cl on dl.enumlabel = cl.enumlabel
    where  dl.enumlabel is null
    or     cl.enumlabel is null;

    if v_message is not null then
        raise exception '%', format('Found errors in %I.%I', $1, $2)||chr(10)||v_message;
    end if;
end;
$$;

comment on procedure data_check.check_enum_definition IS $$
Checks pg catalog tables to ensure that a given schema qualified enum matches the definition found
is the codebase.

Arguments:
schema_name:    Schema name to check
enum_name:      Function name to check
labels:         Collection of labels expected for the enum
$$;

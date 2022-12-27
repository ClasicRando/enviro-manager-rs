create or replace procedure audit.audit_table(
    target_table regclass,
    audit_rows boolean default true,
    audit_query_text boolean default true,
    ignored_cols text[] default '{}'::text[]
)
language plpgsql
as $$
declare
    stm_targets text := 'INSERT OR UPDATE OR DELETE OR TRUNCATE';
    _q_txt text;
    _ignored_cols_snip text := ', ' || quote_literal(coalesce($4,'{}'::text[]));
    _schema text;
    _name text;
begin
    if $2 is null or $3 is null then
        raise exception '2nd and 3rd parameters must be non-null';
    end if;

    select n.nspname::text, c.relname::text
    into   _schema, _name
    from   pg_class c
    join   pg_namespace n on c.relnamespace = n.oid
    where  c.oid = target_table::oid;

    execute format('DROP TRIGGER IF EXISTS audit_trigger_row ON %I.%I', _schema, _name);
    execute format('DROP TRIGGER IF EXISTS audit_trigger_stm ON %I.%I', _schema, _name);

    if $2 then
        _q_txt := format(
            'CREATE TRIGGER audit_trigger_row '
            'AFTER INSERT OR UPDATE OR DELETE ON %I.%I '
            'FOR EACH ROW EXECUTE PROCEDURE audit.if_modified_func(' ||
            quote_literal($3) || _ignored_cols_snip || ');',
            _schema,
            _name
        );
        raise notice '%',_q_txt;
        execute _q_txt;
        stm_targets := 'TRUNCATE';
    end if;

    _q_txt := format(
        'CREATE TRIGGER audit_trigger_stm '
        'AFTER ' || stm_targets || ' ON %I.%I '
        'FOR EACH STATEMENT EXECUTE PROCEDURE audit.if_modified_func(' || quote_literal($3) || ');',
        _schema,
        _name
    );
    raise notice '%',_q_txt;
    execute _q_txt;
end;
$$;

comment on procedure audit.audit_table(regclass, boolean, boolean, text[]) IS $$
Add auditing support to a table.

Arguments:
target_table:     Table name, schema qualified if not on search_path
audit_rows:       Record each row change, or only audit at a statement level, default is true (i.e. row level)
audit_query_text: Record the text of the client query that triggered the audit event? default is true
ignored_cols:     Columns to exclude from update diffs, ignore updates that change only ignored cols. default is none
$$;

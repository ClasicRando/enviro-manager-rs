create or replace function audit.if_modified_func()
returns trigger
language plpgsql
security definer
set search_path = pg_catalog, public
as $$
declare
    audit_row audit.logged_actions;
    excluded_cols text[] := ARRAY[]::text[];
begin
    if TG_WHEN != 'AFTER' then
        raise exception 'audit.if_modified_func() may only run as an AFTER trigger';
    end if;

    audit_row := row(
        -1,                                           -- event_id
        TG_TABLE_SCHEMA::text,                        -- schema_name
        TG_TABLE_NAME::text,                          -- table_name
        TG_RELID,                                     -- relation OID for much quicker searches
        session_user::text,                           -- session_user_name
        current_timestamp,							  -- action_tstamp_tx
        statement_timestamp(),                        -- action_tstamp_stm
        clock_timestamp(),                            -- action_tstamp_clk
        txid_current(),                               -- transaction ID
        current_setting('application_name'),          -- client application
        inet_client_addr(),                           -- client_addr
        inet_client_port(),                           -- client_port
        current_query(),                              -- top-level query or queries (if multistatement) from client
        substring(TG_OP,1,1)::audit.audit_action,     -- action
        null, null,                                   -- row_data, changed_fields
        'f',                                          -- statement_only
        nullif(current_setting('em.uid', true),'')    -- em_user_id
    );

    if not TG_ARGV[0]::boolean is distinct from 'f'::boolean then
        audit_row.client_query := null;
    end if;

    if TG_ARGV[1] is not null then
        excluded_cols := TG_ARGV[1]::text[];
    end if;

    if TG_OP = 'UPDATE' and TG_LEVEL = 'ROW' then
        audit_row.row_data := to_jsonb(OLD.*) - excluded_cols;
        select jsonb_object_agg(new_row.key, new_row.value)
        into   audit_row.changed_fields
        from   jsonb_each_text(to_jsonb(NEW)) new_row
        join   jsonb_each_text(audit_row.row_data) old_row on new_row.key = old_row.key
        where  new_row.value is distinct from old_row.value;

        if audit_row.changed_fields = '{}'::jsonb then
            -- All changed fields are ignored. Skip this update.
            return null;
        end if;
    elsif TG_OP = 'DELETE' and TG_LEVEL = 'ROW' then
        audit_row.row_data = to_jsonb(OLD.*) - excluded_cols;
    elsif TG_OP = 'INSERT' and TG_LEVEL = 'ROW' then
        audit_row.row_data = to_jsonb(NEW.*) - excluded_cols;
    elsif TG_LEVEL = 'STATEMENT' and TG_OP in ('INSERT','UPDATE','DELETE','TRUNCATE') then
        audit_row.statement_only := 't';
    else
        raise exception
            '[audit.if_modified_func] - Trigger func added as trigger for unhandled case: %, %',
            TG_OP,
            TG_LEVEL;
            return null;
    end if;
    insert into audit.logged_actions(
        schema_name,
        table_name,
        relid,
        session_user_name,
        action_tstamp_tx,
        action_tstamp_stm,
        action_tstamp_clk,
        transaction_id,
        application_name,
        client_addr,
        client_port,
        client_query,
        action,
        row_data,
        changed_fields,
        statement_only,
        em_uuid
    )
    values (
        audit_row.schema_name,
        audit_row.table_name,
        audit_row.relid,
        audit_row.session_user_name,
        audit_row.action_tstamp_tx,
        audit_row.action_tstamp_stm,
        audit_row.action_tstamp_clk,
        audit_row.transaction_id,
        audit_row.application_name,
        audit_row.client_addr,
        audit_row.client_port,
        audit_row.client_query,
        audit_row.action,
        audit_row.row_data,
        audit_row.changed_fields,
        audit_row.statement_only,
        audit_row.em_uuid
    );
    return null;
end;
$$;

comment on function audit.if_modified_func() is $$
Track changes to a table at the statement and/or row level.

Optional parameters to trigger in CREATE TRIGGER call:

param 0: boolean, whether to log the query text. Default 't'.

param 1: text[], columns to ignore in updates. Default [].

        Updates to ignored cols are omitted from changed_fields.

        Updates with only ignored cols changed are not inserted
        into the audit log.

        Almost all the processing work is still done for updates
        that ignored. If you need to save the load, you need to use
        WHEN clause on the trigger instead.

        No warning or error is issued if ignored_cols contains columns
        that do not exist in the target table. This lets you specify
        a standard set of ignored columns.

There is no parameter to disable logging of values. Add this trigger as
a 'FOR EACH STATEMENT' rather than 'FOR EACH ROW' trigger if you do not
want to log row values.

Note that the user name logged is the login role for the session. The audit trigger
cannot obtain the active role because it is reset by the SECURITY DEFINER invocation
of the audit trigger its self.
$$;

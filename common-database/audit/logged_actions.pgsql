create table if not exists audit.logged_actions (
    event_id bigint primary key generated always as identity,
    schema_name text not null,
    table_name text not null,
    relid oid not null,
    session_user_name text,
    action_tstamp_tx timestamp with time zone not null,
    action_tstamp_stm timestamp with time zone not null,
    action_tstamp_clk timestamp with time zone not null,
    transaction_id bigint,
    application_name text,
    client_addr inet,
    client_port integer,
    client_query text,
    action audit.audit_action not null,
    row_data jsonb,
    changed_fields jsonb,
    statement_only boolean not null,
    em_uuid text
);

comment on table audit.logged_actions is
'History of auditable actions on audited tables, from audit.if_modified_func()';
comment on column audit.logged_actions.event_id is
'Unique identifier for each auditable event';
comment on column audit.logged_actions.schema_name is
'Database schema audited table for this event is in';
comment on column audit.logged_actions.table_name is
'Non-schema-qualified table name of table event occurred in';
comment on column audit.logged_actions.relid is
'Table OID. Changes with drop/create. Get with ''tablename''::regclass';
comment on column audit.logged_actions.session_user_name is
'Login / session user whose statement caused the audited event';
comment on column audit.logged_actions.action_tstamp_tx is
'Transaction start timestamp for tx in which audited event occurred';
comment on column audit.logged_actions.action_tstamp_stm is
'Statement start timestamp for tx in which audited event occurred';
comment on column audit.logged_actions.action_tstamp_clk is
'Wall clock time at which audited event''s trigger call occurred';
comment on column audit.logged_actions.transaction_id is
'Identifier of transaction that made the change. May wrap, but unique paired with action_tstamp_tx.';
comment on column audit.logged_actions.client_addr is
'IP address of client that issued query. Null for unix domain socket.';
comment on column audit.logged_actions.client_port is
'Remote peer IP port address of client that issued query. Undefined for unix socket.';
comment on column audit.logged_actions.client_query is
'Top-level query that caused this auditable event. May be more than one statement.';
comment on column audit.logged_actions.application_name is
'Application name set when this audit event occurred. Can be changed in-session by client.';
comment on column audit.logged_actions.action is
'Action type; I = insert, D = delete, U = update, T = truncate';
comment on column audit.logged_actions.row_data is $$
Record value. Null for statement-level trigger. For INSERT this is the new tuple. For DELETE and
UPDATE it is the old tuple.
$$;
comment on column audit.logged_actions.changed_fields is
'New values of fields changed by UPDATE. Null except for row-level UPDATE events.';
comment on column audit.logged_actions.statement_only is
'''t'' if audit event is from an FOR EACH STATEMENT trigger, ''f'' for FOR EACH ROW';
comment on column audit.logged_actions.em_uuid is
'UUID of the EnviroManager user who execute the change';

create index if not exists logged_actions_relid_idx on audit.logged_actions(relid);
create index if not exists logged_actions_action_tstamp_tx_stm_idx
    on audit.logged_actions(action_tstamp_stm);
create index if not exists logged_actions_action_idx on audit.logged_actions(action);

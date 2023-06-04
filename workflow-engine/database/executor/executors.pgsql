create or replace function executor.executor_updated_cancel()
returns trigger
language plpgsql
as $$
begin
    perform pg_notify('exec_status_'||new.executor_id, 'cancel');
    return new;
end;
$$;

create or replace function executor.executor_updated_shutdown()
returns trigger
language plpgsql
as $$
begin
    perform pg_notify('exec_status_'||new.executor_id, 'shutdown');
    return new;
end;
$$;

create table if not exists executor.executors (
    executor_id bigint primary key generated always as identity,
    pid integer not null,
    username name not null,
    application_name text not null,
    client_addr inet not null,
    client_port integer not null,
    exec_start timestamp without time zone default (now() at time zone 'UTC'),
    exec_end timestamp without time zone,
    status executor.executor_status not null default 'Active'::executor.executor_status,
    error_message text
);

create or replace trigger canceled_event
    before update of status
    on executor.executors
    for each row
    when new.status = 'Canceled'::executor.executor_status
    execute function executor.executor_updated_cancel();

create or replace trigger shutdown_event
    before update of status
    on executor.executors
    for each row
    when new.status = 'Shutdown'::executor.executor_status
    execute function executor.executor_updated_shutdown();

call audit.audit_table('executor.executors');

revoke all on executor.executors from we_web;

comment on table executor.executors is
'Executors registered as working to complete workflow runs';
comment on column executor.executors.executor_id is
'Unique identifier for each executor';
comment on column executor.executors.pid is
'Process ID of the application running the executor';
comment on column executor.executors.username is
'Name of the current user connecting as the executor';
comment on column executor.executors.application_name is
'Name of the application running the executor';
comment on column executor.executors.client_addr is
'IP address of the client connected as the executor';
comment on column executor.executors.client_port is
'Port of the client connected as the executor';
comment on trigger status_event on executor.executors is
'Trigger run during status updates to notify the required listeners of changes';

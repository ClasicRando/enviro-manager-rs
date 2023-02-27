create or replace function workflow_engine.we_executor_status()
returns trigger
language plpgsql
volatile
as $$
begin
    if new.status = 'Canceled'::workflow_engine.executor_status then
        perform pg_notify('exec_status_'||new.executor_id, 'cancel');
    elsif new.status = 'Shutdown'::workflow_engine.executor_status then
        perform pg_notify('exec_status_'||new.executor_id, 'shutdown');
    end if;
    return new;
end;
$$;

create table if not exists workflow_engine.registered_we_executors (
    executor_id bigint primary key generated always as identity,
    pid integer not null,
    username name not null,
    application_name text not null,
    client_addr inet not null,
    client_port integer not null,
    exec_start timestamp without time zone default (now() at time zone 'UTC'),
    exec_end timestamp without time zone,
    status workflow_engine.executor_status not null default 'Active'::workflow_engine.executor_status,
    error_message text
);

drop trigger if exists status_event on workflow_engine.registered_we_executors;
create trigger status_event
    before update of status
    on workflow_engine.registered_we_executors
    for each row
    execute function workflow_engine.we_executor_status();

call audit.audit_table('workflow_engine.registered_we_executors');

revoke all on workflow_engine.registered_we_executors from public;

comment on table workflow_engine.registered_we_executors is
'Executors registered as working to complete workflow runs';
comment on column workflow_engine.registered_we_executors.executor_id is
'Unique identifier for each executor';
comment on column workflow_engine.registered_we_executors.pid is
'Process ID of the application running the executor';
comment on column workflow_engine.registered_we_executors.username is
'Name of the current user connecting as the executor';
comment on column workflow_engine.registered_we_executors.application_name is
'Name of the application running the executor';
comment on column workflow_engine.registered_we_executors.client_addr is
'IP address of the client connected as the executor';
comment on column workflow_engine.registered_we_executors.client_port is
'Port of the client connected as the executor';
comment on trigger status_event on workflow_engine.registered_we_executors is
'Trigger run during status updates to notify the required listeners of changes';

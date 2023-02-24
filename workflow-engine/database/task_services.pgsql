create table if not exists workflow_engine.task_services (
    service_id bigint primary key generated always as identity,
    name text not null check(data_check.check_not_blank_or_empty(name)) unique,
    base_url text not null check(data_check.check_not_blank_or_empty(base_url))
);

call audit.audit_table('workflow_engine.task_services');

revoke all on workflow_engine.task_services from public;

comment on table workflow_engine.task_services is
'All services that provide remote task execution';
comment on column workflow_engine.task_services.service_id is
'Unique identifier for each service';
comment on column workflow_engine.task_services.name is
'Alias given to the service. Must be unique';
comment on column workflow_engine.task_services.base_url is $$
Base url to connect to the service. Tasks extend this to url to execute specific tasks of a
service
$$;

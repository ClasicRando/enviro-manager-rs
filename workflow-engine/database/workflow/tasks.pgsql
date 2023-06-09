create table if not exists workflow.tasks (
    task_id bigint primary key generated always as identity,
    name text not null check(data_check.check_not_blank_or_empty(name)),
    description text not null check(data_check.check_not_blank_or_empty(description)),
    task_service_id bigint references workflow.task_services match simple
        on delete restrict
        on update cascade,
    url text not null check(data_check.check_not_blank_or_empty(url)),
    constraint name_service_unq unique(name, task_service_id),
    constraint url_service_unq unique(url, task_service_id)
);

call audit.audit_table('workflow.tasks');

comment on table workflow.tasks is
'All tasks executable by the workflow engine';
comment on column workflow.tasks.task_id is
'Unique identifier for each task';
comment on column workflow.tasks.name is
'Alias given to the task';
comment on column workflow.tasks.description is
'Brief description of the task and what it completes';
comment on column workflow.tasks.task_service_id is
'Id of the service hosting this task';
comment on column workflow.tasks.url is
'Extension url to execute the task on the parent service';
comment on constraint name_service_unq on workflow.tasks is
'Ensures that for each service, a name is unique';
comment on constraint url_service_unq on workflow.tasks is
'Ensures that for each service, a url extension is unique';
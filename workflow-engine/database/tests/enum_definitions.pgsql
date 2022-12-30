declare
    v_schema text := 'workflow_engine';
begin
    call data_check.check_enum_definition(v_schema,'executor_status','{Active,Canceled,Shutdown}');
    call data_check.check_enum_definition(v_schema,'job_type','{Scheduled,Interval}');
    call data_check.check_enum_definition(v_schema,'task_status','{Waiting,Running,Paused,Failed,Rule Broken,Complete,Canceled}');
    call data_check.check_enum_definition(v_schema,'workflow_run_status','{Waiting,Scheduled,Running,Paused,Failed,Complete,Canceled}');
end;
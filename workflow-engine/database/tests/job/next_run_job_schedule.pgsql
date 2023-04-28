declare
    v_schedule job.schedule_entry[];
    v_start_of_week timestamp without time zone := date_trunc('week', now() at time zone 'UTC');
    v_next_monday timestamp without time zone := v_start_of_week + interval '7 days';
    v_start_of_day timestamp without time zone := date_trunc('day', now() at time zone 'UTC');
    v_next_morning timestamp without time zone := v_start_of_day + interval '1 day';
    v_result timestamp without time zone;
begin
    assert
        job.next_run_job_schedule(null) is null,
        'A null job schedule should return null';

    v_schedule := array[]::job.schedule_entry[];
    assert
        job.next_run_job_schedule(v_schedule) is null,
        'An empty job schedule should return null';

    v_schedule := array[row(1,'00:00:00')::job.schedule_entry];
    v_result := job.next_run_job_schedule(v_schedule);
    assert
        v_result = v_next_monday,
        format(
            'A job schedule with 1 entry at Monday, 12:00am should return %s but got %s',
            v_next_monday,
            v_result
        );

    v_schedule := array[
        row(1,'00:00:00')::job.schedule_entry,
        row(2,'00:00:00')::job.schedule_entry,
        row(3,'00:00:00')::job.schedule_entry,
        row(4,'00:00:00')::job.schedule_entry,
        row(5,'00:00:00')::job.schedule_entry,
        row(6,'00:00:00')::job.schedule_entry,
        row(7,'00:00:00')::job.schedule_entry
    ];
    v_result := job.next_run_job_schedule(v_schedule);
    assert
        v_result = v_next_morning,
        format(
            'A daily job schedule at 12:00am should return %s but got %s',
            v_next_morning,
            v_result
        );
end;
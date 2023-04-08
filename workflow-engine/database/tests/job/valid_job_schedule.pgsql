declare
    v_schedule job.schedule_entry[];
begin
    assert
        not job.valid_job_schedule(null),
        'A null job schedule should return false';

    v_schedule := array[]::job.schedule_entry[];
    assert
        not job.valid_job_schedule(v_schedule),
        'An empty job schedule should return false';

    v_schedule := array[
        row(1, '00:00:00')::job.schedule_entry,
        row(8, '00:00:00')::job.schedule_entry
    ];
    assert
        not job.valid_job_schedule(v_schedule),
        'A job schedule with a day of the week greater than 7 should return false';

    v_schedule := array[
        row(-1, '00:00:00')::job.schedule_entry,
        row(2, '00:00:00')::job.schedule_entry
    ];
    assert
        not job.valid_job_schedule(v_schedule),
        'A job schedule with a day of the week less than 1 should return false';

    v_schedule := array[
        row(1, '00:00:00')::job.schedule_entry,
        row(1, '00:00:00')::job.schedule_entry,
        row(2, '00:00:00')::job.schedule_entry
    ];
    assert
        not job.valid_job_schedule(v_schedule),
        'A job schedule with duplicate entries should return false';

    v_schedule := array[
        row(1, '00:00:00')::job.schedule_entry,
        row(2, '00:00:00')::job.schedule_entry
    ];
    assert
        job.valid_job_schedule(v_schedule),
        'A valid job schedule should return true';
end;

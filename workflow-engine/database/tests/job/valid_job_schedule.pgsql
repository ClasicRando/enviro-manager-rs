begin
    assert
        not job.valid_job_schedule(null),
        'A null job schedule should return false';
    assert
        not job.valid_job_schedule(array[]::job.schedule_entry[]),
        'An empty job schedule should return false';
    assert
        not job.valid_job_schedule(
            array[
                row(1, '00:00:00')::job.schedule_entry,
                row(8, '00:00:00')::job.schedule_entry
            ]
        ),
        'A job schedule with a day of the week greater than 7 should return false';
    assert
        not job.valid_job_schedule(
            array[
                row(-1, '00:00:00')::job.schedule_entry,
                row(2, '00:00:00')::job.schedule_entry
            ]
        ),
        'A job schedule with a day of the week less than 1 should return false';
    assert
        job.valid_job_schedule(
            array[
                row(1, '00:00:00')::job.schedule_entry,
                row(2, '00:00:00')::job.schedule_entry
            ]
        ),
        'A valid job schedule should return true';
end;

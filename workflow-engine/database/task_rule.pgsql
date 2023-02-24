if not data_check.type_exists('workflow_engine','task_rule') then
    create type workflow_engine.task_rule as
    (
        name text,
        failed boolean,
        message text
    );
end if;

comment on type workflow_engine.task_rule IS $$
Describes rules checked during task execution.

Attributes:
name:
    Alias given to the rule to describe what was checked
failed:
    flag to indicate if the rule check was not verified
message:
    Feedback to users. Usually non-null when broken but can be populated even on success
$$;

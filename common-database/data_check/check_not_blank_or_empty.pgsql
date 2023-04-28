create or replace function data_check.check_not_blank_or_empty(
    text
) returns boolean
language plpgsql
immutable
as $$
begin
    return coalesce($1,'x') !~ '^\s*$';
end;
$$;

comment on function data_check.check_not_blank_or_empty IS $$
Check to confirm text value is not empty or only containing whitespace. Returns false when either
criteria is met. Null values return true.
$$;

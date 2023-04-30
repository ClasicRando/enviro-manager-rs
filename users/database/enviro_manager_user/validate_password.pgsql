create procedure enviro_manager_user.validate_password(password text)
language plpgsql
as $$
begin
    if $1 is null or not data_check.check_not_blank_or_empty($1) then
        raise exception 'password does meet the requirements. Must not be null or an empty string';
    end if;
    if $1 !~ '[A-Z]' then
        raise exception 'password does meet the requirements. Must contain at least 1 uppercase character.';
    end if;
    if $1 !~ '\d' then
        raise exception 'password does meet the requirements. Must contain at least 1 digit character.';
    end if;
    if $1 !~ '\W' then
        raise exception 'password does meet the requirements. Must contain at least 1 non-alphanumeric character.';
    end if;
end;
$$;

comment on procedure enviro_manager_user.validate_password IS $$
Raises an exception if the password does not meet the requirements. Must contains at least 1
uppercase character, digit and non-alphanumeric character to be validated.

Arguments:
password:
    Password value that the user is trying to create or update for the user.
$$;

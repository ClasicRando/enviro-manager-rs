declare
    v_msg text;
    v_unexpected_exit boolean;
begin
    -- Must not be null
    v_unexpected_exit := true;
    begin
        call users.validate_password(null);
    exception
        when others then
            get stacked diagnostics
                v_msg = message_text;
            assert
                v_msg = 'password does meet the requirements. Must not be null or an empty string',
                format(
                    'An exception must be raised when trying to validate a password. Error %s',
                    v_msg
                );
            v_unexpected_exit := false;
    end;
    
    assert not v_unexpected_exit, 'Unexpected exit of block without capturing an exception';

    -- Must not be empty
    v_unexpected_exit := true;
    begin
        call users.validate_password('');
    exception
        when others then
            get stacked diagnostics
                v_msg = message_text;
            assert
                v_msg = 'password does meet the requirements. Must not be null or an empty string',
                format(
                    'An exception must be raised when trying to validate a password. Error %s',
                    v_msg
                );
            v_unexpected_exit := false;
    end;
    
    assert not v_unexpected_exit, 'Unexpected exit of block without capturing an exception';

    -- Must contain uppercase
    v_unexpected_exit := true;
    begin
        call users.validate_password('lowercase');
    exception
        when others then
            get stacked diagnostics
                v_msg = message_text;
            assert
                v_msg = 'password does meet the requirements. Must contain at least 1 uppercase character.',
                format(
                    'An exception must be raised when trying to validate a password. Error %s',
                    v_msg
                );
            v_unexpected_exit := false;
    end;
    
    assert not v_unexpected_exit, 'Unexpected exit of block without capturing an exception';

    -- Must contain at least 1 digit character
    v_unexpected_exit := true;
    begin
        call users.validate_password('MissingDigit');
    exception
        when others then
            get stacked diagnostics
                v_msg = message_text;
            assert
                v_msg = 'password does meet the requirements. Must contain at least 1 digit character.',
                format(
                    'An exception must be raised when trying to validate a password. Error %s',
                    v_msg
                );
            v_unexpected_exit := false;
    end;
    
    assert not v_unexpected_exit, 'Unexpected exit of block without capturing an exception';

    -- Must contain at least 1 non-alphanumeric character
    v_unexpected_exit := true;
    begin
        call users.validate_password('MissingNonAlphaNumeric1');
    exception
        when others then
            get stacked diagnostics
                v_msg = message_text;
            assert
                v_msg = 'password does meet the requirements. Must contain at least 1 non-alphanumeric character.',
                format(
                    'An exception must be raised when trying to validate a password. Error %s',
                    v_msg
                );
            v_unexpected_exit := false;
    end;
    
    assert not v_unexpected_exit, 'Unexpected exit of block without capturing an exception';

    -- This test should not raise an exception
    call users.validate_password('C0rrectPa$$word');
end;

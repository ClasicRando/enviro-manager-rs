if not data_check.type_exists('audit','audit_action') then
    create type audit.audit_action as enum('I','D','U','T');
end if;

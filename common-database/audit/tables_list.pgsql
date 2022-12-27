create view audit.tableslist as
select distinct triggers.trigger_schema as schema, triggers.event_object_table AS auditedtable
from   information_schema.triggers
where  triggers.trigger_name::text in ('audit_trigger_row'::text, 'audit_trigger_stm'::text)
order by 1, 2;

comment on view audit.tableslist is $$
View showing all tables with auditing set up. Ordered by schema, then table.
$$;

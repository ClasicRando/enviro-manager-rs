create or replace function enviro_manager_user.get_current_em_uid()
returns bigint
stable
language sql
as $$
select nullif(current_setting('em.uid', true),'')::bigint;
$$;

comment on function enviro_manager_user.get_current_em_uid IS $$
Fetch the current setting for 'em.uid' as a bigint. This property is set when privileges are
required for an action.
$$;

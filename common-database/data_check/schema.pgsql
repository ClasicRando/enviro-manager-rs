create schema if not exists data_check;
revoke all on schema data_check from public;

comment on schema data_check is 'Common functions and procedures to verify data integrity';

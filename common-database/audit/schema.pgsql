-- Custom implmentation of audit-trigger project, https://github.com/2ndQuadrant/audit-trigger
create schema if not exists audit;
revoke all on schema audit from public;

comment on schema audit is 'Out-of-table audit/history logging tables and trigger functions';

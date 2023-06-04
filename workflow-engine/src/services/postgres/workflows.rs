use common::{api::ApiRequestValidator, database::postgres::Postgres, error::EmResult};
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    PgPool,
};

use crate::services::workflows::{
    Workflow, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest, WorkflowRequestValidator,
    WorkflowTask, WorkflowTaskRequest, WorkflowsService,
};

impl PgHasArrayType for WorkflowTask {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_task")
    }
}

impl PgHasArrayType for WorkflowTaskRequest {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workflow_task_request")
    }
}

/// Postgres implementation of [WorkflowsService]
#[derive(Clone)]
pub struct PgWorkflowsService {
    pool: PgPool,
}

impl WorkflowsService for PgWorkflowsService {
    type Database = Postgres;
    type RequestValidator = WorkflowRequestValidator;

    fn create(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_workflow(&self, request: &WorkflowRequest) -> EmResult<Workflow> {
        Self::RequestValidator::validate(request)?;
        let workflow_id = sqlx::query_scalar("select workflow.create_workflow($1,$2)")
            .bind(&request.name)
            .bind(&request.tasks)
            .fetch_one(&self.pool)
            .await?;
        match self.read_one(&workflow_id).await {
            Ok(Some(workflow)) => Ok(workflow),
            Ok(None) => Err(sqlx::Error::RowNotFound.into()),
            Err(error) => Err(error),
        }
    }

    async fn read_one(&self, workflow_id: &WorkflowId) -> EmResult<Option<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select w.workflow_id, w.name, w.tasks
            from workflow.v_workflows w
            where w.workflow_id = $1"#,
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    async fn read_many(&self) -> EmResult<Vec<Workflow>> {
        let result = sqlx::query_as(
            r#"
            select w.workflow_id, w.name, w.tasks
            from workflow.v_workflows w"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn deprecate(&self, request: &WorkflowDeprecationRequest) -> EmResult<WorkflowId> {
        sqlx::query("call workflow.deprecate_workflow($1,$2)")
            .bind(request.workflow_id)
            .bind(request.new_workflow_id)
            .execute(&self.pool)
            .await?;
        Ok(request.workflow_id)
    }
}

// #[cfg(test)]
// mod test {
//     use common::{
//         database::{connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder},
//         error::EmResult,
//     };
//     use sqlx::PgPool;
//
//     use crate::{
//         database::db_options,
//         services::{
//             postgres::workflows::PgWorkflowsService,
//             tasks::TaskId,
//             workflows::{
//                 Workflow, WorkflowDeprecationRequest, WorkflowId, WorkflowRequest,
//                 WorkflowTaskRequest,
//             },
//         },
//         WorkflowsService,
//     };
//
//     async fn clean_test_workflow(workflow_name: &str, pool: &PgPool) -> EmResult<()> {
//         sqlx::query(
//             r#"
//             with workflows as (
//                 delete from task.workflow_tasks wt
//                 using workflow.workflows w
//                 where
//                     w.name = $1
//                     and wt.workflow_id = w.workflow_id
//                 returning wt.workflow_id
//             )
//             delete from workflow.workflows w1
//             using (
//                 select w1.workflow_id
//                 from workflows w1
//                 union
//                 select w2.workflow_id
//                 from workflow.workflows w2
//                 where w2.name = $2
//             ) w2
//             where w1.workflow_id = w2.workflow_id"#,
//         )
//         .bind(workflow_name)
//         .bind(workflow_name)
//         .execute(pool)
//         .await?;
//         Ok(())
//     }
//
//     async fn reset_base_test_workflow(pool: &PgPool) -> EmResult<()> {
//         sqlx::query(
//             r#"
//             update workflow.workflows
//             set
//                 is_deprecated = false,
//                 new_workflow = null
//             where workflow_id = 1"#,
//         )
//         .execute(pool)
//         .await?;
//         Ok(())
//     }
//
//     #[sqlx::test]
//     async fn create() -> EmResult<()> {
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let service = PgWorkflowsService::create(&pool);
//         let workflow_name = "Create test";
//         let task_id = TaskId::from(1);
//         let parameters = None;
//
//         let request = WorkflowRequest {
//             name: String::from(workflow_name),
//             tasks: vec![WorkflowTaskRequest {
//                 task_id,
//                 parameters,
//             }],
//         };
//
//         let workflow = service.create_workflow(&request).await?;
//
//         assert_eq!(workflow.name, workflow_name);
//         assert_eq!(workflow.tasks.len(), 1);
//         assert_eq!(workflow.tasks[0].task_id, task_id);
//
//         clean_test_workflow(workflow_name, &pool).await?;
//         let workflow = service.read_one(&workflow.workflow_id).await?;
//         assert!(workflow.is_none());
//
//         Ok(())
//     }
//
//     #[sqlx::test]
//     async fn read_one_should_return_some_when_record_exists() -> EmResult<()> {
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let service = PgWorkflowsService::create(&pool);
//         let workflow_id = WorkflowId::from(1);
//         let workflow_name = "test";
//         let task_id = TaskId::from(1);
//
//         let Some(workflow) = service.read_one(&workflow_id).await? else {
//             panic!("Record not found")
//         };
//
//         assert_eq!(workflow.name, workflow_name);
//         assert_eq!(workflow.tasks.len(), 1);
//         assert_eq!(workflow.tasks[0].task_id, task_id);
//
//         Ok(())
//     }
//
//     #[sqlx::test]
//     async fn read_one_should_return_none_when_record_exists() -> EmResult<()> {
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let service = PgWorkflowsService::create(&pool);
//         let workflow_id = WorkflowId::from(0);
//
//         let result = service.read_one(&workflow_id).await?;
//
//         assert!(result.is_none());
//
//         Ok(())
//     }
//
//     #[sqlx::test]
//     async fn read_many() -> EmResult<()> {
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let service = PgWorkflowsService::create(&pool);
//
//         let workflows = service.read_many().await?;
//
//         assert!(!workflows.is_empty());
//
//         Ok(())
//     }
//
//     #[sqlx::test]
//     async fn deprecate_workflow_with_no_new_workflow() -> EmResult<()> {
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let service = PgWorkflowsService::create(&pool);
//
//         let workflow_name = "deprecate no new workflow test";
//         let task_id = TaskId::from(1);
//         let parameters = None;
//
//         let request = WorkflowRequest {
//             name: String::from(workflow_name),
//             tasks: vec![WorkflowTaskRequest {
//                 task_id,
//                 parameters,
//             }],
//         };
//
//         let Workflow {
//             workflow_id: created_workflow_id,
//             ..
//         } = service.create_workflow(&request).await?;
//
//         let request = WorkflowDeprecationRequest {
//             workflow_id: created_workflow_id,
//             new_workflow_id: None,
//         };
//
//         let return_workflow_id = service.deprecate(&request).await?;
//         let (is_deprecated, new_workflow_id): (bool, Option<i64>) = sqlx::query_as(
//             r#"
//             select w.is_deprecated, w.new_workflow
//             from workflow.workflows w
//             where w.workflow_id = $1"#,
//         )
//         .bind(created_workflow_id)
//         .fetch_one(&pool)
//         .await?;
//
//         assert_eq!(created_workflow_id, return_workflow_id);
//         assert_eq!(new_workflow_id, None);
//         assert!(is_deprecated);
//
//         reset_base_test_workflow(&pool).await?;
//
//         Ok(())
//     }
//
//     #[sqlx::test]
//     async fn deprecate_workflow_with_new_workflow() -> EmResult<()> {
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let service = PgWorkflowsService::create(&pool);
//         let workflow_id = WorkflowId::from(1);
//         let new_workflow_name = "deprecate workflow new workflow test";
//         let task_id = TaskId::from(1);
//         let parameters = None;
//
//         let request = WorkflowRequest {
//             name: String::from(new_workflow_name),
//             tasks: vec![WorkflowTaskRequest {
//                 task_id,
//                 parameters,
//             }],
//         };
//
//         let workflow = service.create_workflow(&request).await?;
//         let Workflow {
//             workflow_id: created_workflow_id,
//             ..
//         } = &workflow;
//
//         let request = WorkflowDeprecationRequest {
//             workflow_id,
//             new_workflow_id: Some(*created_workflow_id),
//         };
//
//         let return_workflow_id = service.deprecate(&request).await?;
//         let (is_deprecated, new_workflow_id): (bool, Option<WorkflowId>) = sqlx::query_as(
//             r#"
//             select w.is_deprecated, w.new_workflow
//             from workflow.workflows w
//             where w.workflow_id = $1"#,
//         )
//         .bind(workflow_id)
//         .fetch_one(&pool)
//         .await?;
//
//         assert_eq!(workflow_id, return_workflow_id);
//         assert_eq!(new_workflow_id, Some(*created_workflow_id));
//         assert!(is_deprecated);
//
//         clean_test_workflow(new_workflow_name, &pool).await?;
//         let workflow = service.read_one(created_workflow_id).await?;
//         assert!(workflow.is_none());
//
//         reset_base_test_workflow(&pool).await?;
//
//         Ok(())
//     }
// }

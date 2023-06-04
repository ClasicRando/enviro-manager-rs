use common::{
    database::postgres::Postgres,
    error::{EmError, EmResult},
};
use sqlx::PgPool;

use crate::services::tasks::{Task, TaskId, TaskRequest, TaskRequestValidator, TaskService};

/// Postgres implementation of [TaskService]
#[derive(Clone)]
pub struct PgTasksService {
    pool: PgPool,
}

impl TaskService for PgTasksService {
    type Database = Postgres;
    type RequestValidator = TaskRequestValidator;

    fn create(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn create_task(&self, request: &TaskRequest) -> EmResult<Task> {
        let task_id: TaskId = sqlx::query_scalar("select task.create_task($1,$2,$3,$4)")
            .bind(&request.name)
            .bind(&request.description)
            .bind(request.task_service_id)
            .bind(&request.url)
            .fetch_one(&self.pool)
            .await?;
        self.read_one(&task_id).await
    }

    async fn read_one(&self, task_id: &TaskId) -> EmResult<Task> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from task.v_tasks
            where task_id = $1"#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;
        result.map_or_else(
            || {
                Err(EmError::MissingRecord {
                    pk: task_id.to_string(),
                })
            },
            Ok,
        )
    }

    async fn read_many(&self) -> EmResult<Vec<Task>> {
        let result = sqlx::query_as(
            r#"
            select task_id, name, description, url, task_service_name
            from task.v_tasks"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn update(&self, task_id: &TaskId, request: &TaskRequest) -> EmResult<Task> {
        sqlx::query("call task.update_task($1,$2,$3,$4,$5)")
            .bind(task_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(request.task_service_id)
            .bind(&request.url)
            .execute(&self.pool)
            .await?;
        self.read_one(task_id).await
    }
}

// #[cfg(test)]
// mod test {
//     use common::database::{
//         connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder,
//     };
//
//     use super::{PgTasksService, TaskRequest, TaskService};
//     use crate::database::db_options;
//
//     #[sqlx::test]
//     async fn task() -> Result<(), Box<dyn std::error::Error>> {
//         let task_name = "Create Task Test";
//         let task_description = "Test task created as integration testing.";
//         let task_service_id = 1_i64;
//         let task_url = r"test2";
//
//         let request = TaskRequest {
//             name: task_name.to_string(),
//             description: task_description.to_string(),
//             task_service_id,
//             url: task_url.to_string(),
//         };
//
//         let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
//         let (task_service_name, service_url): (String, String) =
//             sqlx::query_as("select name, base_url from task.task_services where service_id = $1")
//                 .bind(task_service_id)
//                 .fetch_one(&pool)
//                 .await?;
//         let task_url_full = format!("{}\\{}", service_url, task_url);
//         sqlx::query("delete from task.tasks where name = $1")
//             .bind(task_name)
//             .execute(&pool)
//             .await?;
//
//         let tasks_service = PgTasksService::create(&pool);
//
//         let task = tasks_service.create_task(&request).await?;
//
//         assert_eq!(task.name, task_name);
//         assert_eq!(task.description, task_description);
//         assert_eq!(task.task_service_name, task_service_name);
//         assert_eq!(task.url, task_url_full);
//
//         let task = tasks_service.read_one(&task.task_id).await?;
//
//         assert_eq!(task.name, task_name);
//         assert_eq!(task.description, task_description);
//         assert_eq!(task.task_service_name, task_service_name);
//         assert_eq!(task.url, task_url_full);
//
//         let count: i64 = sqlx::query_scalar("select count(0) from task.tasks")
//             .fetch_one(&pool)
//             .await?;
//         let tasks = tasks_service.read_many().await?;
//
//         assert_eq!(count as usize, tasks.len());
//
//         let new_description = "New Task Description";
//         let request = TaskRequest {
//             description: new_description.to_string(),
//             ..request
//         };
//         let task = tasks_service.update(&task.task_id, &request).await?;
//
//         assert_eq!(task.name, task_name);
//         assert_eq!(task.description, new_description);
//         assert_eq!(task.task_service_name, task_service_name);
//         assert_eq!(task.url, task_url_full);
//
//         Ok(())
//     }
// }

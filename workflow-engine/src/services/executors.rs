use chrono::NaiveDateTime;
use log::error;
use rocket::request::FromParam;
use serde::Serialize;
use sqlx::{postgres::PgListener, types::ipnetwork::IpNetwork, PgPool};

use crate::error::{Error as WEError, Result as WEResult};

use super::workflow_runs::WorkflowRunId;

#[derive(sqlx::Type, Serialize, PartialEq, Debug)]
#[sqlx(type_name = "executor_status")]
pub enum ExecutorStatus {
    Active,
    Canceled,
    Shutdown,
}

fn serialize_ipnetwork<S>(addr: &IpNetwork, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.collect_str(addr)
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Executor {
    executor_id: i64,
    pid: i32,
    username: String,
    application_name: String,
    #[serde(serialize_with = "serialize_ipnetwork")]
    client_addr: IpNetwork,
    client_port: i32,
    exec_start: NaiveDateTime,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_active: Option<bool>,
    #[sqlx(default, rename = "wr_count")]
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_run_count: Option<i64>,
}

#[derive(sqlx::Type, Clone)]
#[sqlx(transparent)]
pub struct ExecutorId(i64);

impl<'a> FromParam<'a> for ExecutorId {
    type Error = WEError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(param.parse::<i64>()?))
    }
}

impl std::fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct ExecutorsService {
    pool: PgPool,
}

impl ExecutorsService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn register_executor(&self) -> WEResult<ExecutorId> {
        let executor_id = sqlx::query_scalar("select executor.register_executor()")
            .fetch_one(&self.pool)
            .await?;
        Ok(executor_id)
    }

    pub async fn read_one(&self, executor_id: &ExecutorId) -> WEResult<Option<Executor>> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_executors e
            where e.executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_status(&self, executor_id: &ExecutorId) -> WEResult<Option<ExecutorStatus>> {
        let result = sqlx::query_scalar(
            r#"
            select e.status
            from executor.executors e
            where e.executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_many(&self) -> WEResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_executors e"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn read_active(&self) -> WEResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_active_executors e"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    pub async fn next_workflow_run(
        &self,
        executor_id: &ExecutorId,
    ) -> WEResult<Option<WorkflowRunId>> {
        let workflow_run_id: Option<WorkflowRunId> =
            sqlx::query_scalar("call workflow.process_next_workflow($1,$2)")
                .bind(executor_id)
                .bind(None::<i64>)
                .fetch_one(&self.pool)
                .await?;
        Ok(workflow_run_id)
    }

    pub async fn shutdown(&self, executor_id: &ExecutorId) -> WEResult<Option<Executor>> {
        sqlx::query("call executor.shutdown_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    pub async fn cancel(&self, executor_id: &ExecutorId) -> WEResult<Option<Executor>> {
        sqlx::query("call executor.cancel_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    pub async fn close(&self, executor_id: &ExecutorId, is_cancelled: bool) -> WEResult<()> {
        sqlx::query("call executor.close_executor($1,$2)")
            .bind(executor_id)
            .bind(is_cancelled)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn post_error(&self, executor_id: &ExecutorId, error: WEError) -> WEResult<()> {
        let message = format!("{}", error);
        let sql_result = sqlx::query("call executor.post_executor_error_message($1,$2)")
            .bind(executor_id)
            .bind(&message)
            .execute(&self.pool)
            .await;
        if let Err(error) = sql_result {
            error!(
                "Error while attempting to post executor error.\n{:?}",
                error
            );
        }
        error!("Executor fatal error. {}", message);
        Ok(())
    }

    pub async fn clean_executors(&self) -> WEResult<()> {
        sqlx::query("call executor.clean_executors()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn status_listener(&self, executor_id: &ExecutorId) -> WEResult<PgListener> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener
            .listen(&format!("exec_status_{}", executor_id))
            .await?;
        Ok(listener)
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{database::utilities::create_test_db_pool, error::Error as WEError};

    use super::{ExecutorStatus, ExecutorsService};

    #[sqlx::test]
    async fn create_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        if executor_service.read_one(&executor_id).await?.is_none() {
            panic!("Failed to `read_one`");
        };

        let Some(executor_status) = executor_service.read_status(&executor_id).await? else {
            panic!("Failed to `read_status`");
        };
        assert_eq!(executor_status, ExecutorStatus::Active);
        Ok(())
    }

    #[sqlx::test]
    async fn cancel_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        if executor_service.cancel(&executor_id).await?.is_none() {
            panic!("Failed to `cancel`");
        };
        let Some(executor_status) = executor_service.read_status(&executor_id).await? else {
            panic!("Failed to `read_status`");
        };
        assert_eq!(executor_status, ExecutorStatus::Canceled);

        Ok(())
    }

    #[sqlx::test]
    async fn shutdown_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        if executor_service.shutdown(&executor_id).await?.is_none() {
            panic!("Failed to `shutdown`");
        };
        let Some(executor_status) = executor_service.read_status(&executor_id).await? else {
            panic!("Failed to `read_status`");
        };
        assert_eq!(executor_status, ExecutorStatus::Shutdown);

        Ok(())
    }

    #[sqlx::test]
    async fn post_error() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService::new(&pool);

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let error = WEError::Generic(String::from("Executor 'post_error' test"));
        let error_message = error.to_string();
        executor_service.post_error(&executor_id, error).await?;

        let query = indoc! {
            r#"
            select e.error_message
            from executor.executors e
            where e.executor_id = $1"#
        };
        let message: String = sqlx::query_scalar(query)
            .bind(&executor_id)
            .fetch_one(&pool)
            .await?;
        assert_eq!(message, error_message);

        Ok(())
    }

    #[sqlx::test]
    async fn status_listener() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService::new(&pool);

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let mut listener = executor_service.status_listener(&executor_id).await?;

        let message = "Test";
        sqlx::query(&format!(
            "NOTIFY exec_status_{}, '{}'",
            executor_id, message
        ))
        .execute(&pool)
        .await?;
        let notification = listener.recv().await?;

        assert_eq!(message, notification.payload());

        Ok(())
    }

    #[sqlx::test]
    async fn clean_executors() -> Result<(), Box<dyn std::error::Error>> {
        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService::new(&pool);

        let inactive_executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        drop(executor_service);
        pool.close().await;
        drop(pool);

        let pool = create_test_db_pool().await?;
        let executor_service = ExecutorsService::new(&pool);
        executor_service.clean_executors().await?;

        let Some(status) = executor_service.read_status(&inactive_executor_id).await? else {
            panic!("Could not `read_status`");
        };

        assert_eq!(
            status,
            ExecutorStatus::Canceled,
            "Status was not Canceled for executor_id = {}",
            inactive_executor_id
        );

        Ok(())
    }
}

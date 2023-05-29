use common::{
    database::connection::finalize_transaction,
    error::{EmError, EmResult},
};
use log::error;
use sqlx::{postgres::PgListener, PgPool, Postgres, Transaction};

use crate::{
    database::listener::PgChangeListener,
    executor::utilities::ExecutorStatusUpdate,
    services::{
        executors::{Executor, ExecutorId, ExecutorStatus},
        workflow_runs::WorkflowRunId,
    },
    ExecutorService,
};

/// Postgresql implementation of the [ExecutorService]. Wraps a [PgPool] and provides interaction
/// methods for the API and [Executor][crate::executor::Executor] instances.
#[derive(Clone)]
pub struct PgExecutorService {
    pool: PgPool,
}

impl PgExecutorService {
    /// Start a workflow run by executing the named procedure. Takes ownership of the `transaction`
    /// and completes the transaction before exiting.
    /// # Errors
    /// This function will return an error if the `start_workflow_run` procedure fails or the an
    /// error is returning completing the `transaction`
    async fn start_workflow_run<'c>(
        executor_id: &ExecutorId,
        workflow_run_id: &WorkflowRunId,
        mut transaction: Transaction<'c, Postgres>,
    ) -> EmResult<()> {
        let result = sqlx::query("call executor.start_workflow_run($1, $2)")
            .bind(workflow_run_id)
            .bind(executor_id)
            .execute(&mut transaction)
            .await;
        finalize_transaction(result, transaction).await?;
        Ok(())
    }

    /// Complete a workflow run by executing the named procedure. Takes ownership of the
    /// `transaction` and completes the transaction before exiting.
    /// # Errors
    /// This function will return an error if the `complete_workflow_run` procedure fails or the an
    /// error is returning completing the `transaction`
    async fn complete_workflow_run<'c>(
        workflow_run_id: &WorkflowRunId,
        mut transaction: Transaction<'c, Postgres>,
    ) -> EmResult<()> {
        let result = sqlx::query("call executor.complete_workflow_run($1)")
            .bind(workflow_run_id)
            .execute(&mut transaction)
            .await;
        finalize_transaction(result, transaction).await?;
        Ok(())
    }
}

impl ExecutorService for PgExecutorService {
    type Database = Postgres;
    type Listener = PgChangeListener<ExecutorStatusUpdate>;

    fn create(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    async fn register_executor(&self) -> EmResult<ExecutorId> {
        let executor_id = sqlx::query_scalar("select executor.register_executor()")
            .fetch_one(&self.pool)
            .await?;
        Ok(executor_id)
    }

    async fn read_one(&self, executor_id: &ExecutorId) -> EmResult<Executor> {
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
        match result {
            Some(executor) => Ok(executor),
            None => Err(EmError::MissingRecord {
                pk: executor_id.to_string(),
            }),
        }
    }

    async fn read_status(&self, executor_id: &ExecutorId) -> EmResult<ExecutorStatus> {
        let result = sqlx::query_scalar(
            r#"
            select e.status
            from executor.v_executors e
            where e.executor_id = $1"#,
        )
        .bind(executor_id)
        .fetch_optional(&self.pool)
        .await?;
        match result {
            Some(status) => Ok(status),
            None => Err(EmError::MissingRecord {
                pk: executor_id.to_string(),
            }),
        }
    }

    async fn read_many(&self) -> EmResult<Vec<Executor>> {
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

    async fn read_active(&self) -> EmResult<Vec<Executor>> {
        let result = sqlx::query_as(
            r#"
            select
                e.executor_id, e.pid, e.username, e.application_name, e.client_addr, e.client_port,
                e.exec_start, e.session_active, e.wr_count
            from executor.v_executors e
            where e.status = 'Active'::executor.executor_status"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    async fn next_workflow_run(&self, executor_id: &ExecutorId) -> EmResult<Option<WorkflowRunId>> {
        let mut transaction = self.pool.begin().await?;
        let next_workflow: Option<(WorkflowRunId, bool)> =
            sqlx::query_as("call workflow.next_workflow($1)")
                .bind(executor_id)
                .fetch_optional(&mut transaction)
                .await?;
        let Some((workflow_run_id, is_valid)) = next_workflow else {
            transaction.commit().await?;
            return Ok(None)
        };

        if is_valid {
            Self::start_workflow_run(executor_id, &workflow_run_id, transaction).await?;
        } else {
            Self::complete_workflow_run(&workflow_run_id, transaction).await?;
        }

        Ok(Some(workflow_run_id))
    }

    async fn shutdown(&self, executor_id: &ExecutorId) -> EmResult<Executor> {
        sqlx::query("call executor.shutdown_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    async fn cancel(&self, executor_id: &ExecutorId) -> EmResult<Executor> {
        sqlx::query("call executor.cancel_executor($1)")
            .bind(executor_id)
            .execute(&self.pool)
            .await?;
        self.read_one(executor_id).await
    }

    async fn close(&self, executor_id: &ExecutorId, is_cancelled: bool) -> EmResult<()> {
        sqlx::query("call executor.close_executor($1,$2)")
            .bind(executor_id)
            .bind(is_cancelled)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn post_error(&self, executor_id: &ExecutorId, error: EmError) {
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
    }

    async fn clean_executors(&self) -> EmResult<()> {
        sqlx::query("call executor.clean_executors()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn status_listener(&self, executor_id: &ExecutorId) -> EmResult<Self::Listener> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener
            .listen(&format!("exec_status_{}", executor_id))
            .await?;
        Ok(PgChangeListener::new(listener))
    }
}

#[cfg(test)]
mod test {
    use common::{
        database::{connection::ConnectionBuilder, postgres::connection::PgConnectionBuilder},
        error::EmError,
    };
    use indoc::indoc;

    use super::{ExecutorService, ExecutorStatus, PgExecutorService};
    use crate::{
        database::{db_options, listener::ChangeListener},
        executor::utilities::ExecutorStatusUpdate,
    };

    #[sqlx::test]
    async fn create_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let executor_status = executor_service.read_status(&executor_id).await?;
        assert_eq!(executor_status, ExecutorStatus::Active);
        Ok(())
    }

    #[sqlx::test]
    async fn cancel_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let executor_status = executor_service.read_status(&executor_id).await?;
        assert_eq!(executor_status, ExecutorStatus::Canceled);

        Ok(())
    }

    #[sqlx::test]
    async fn shutdown_executor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService { pool };

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let executor_status = executor_service.read_status(&executor_id).await?;
        assert_eq!(executor_status, ExecutorStatus::Shutdown);

        Ok(())
    }

    #[sqlx::test]
    async fn post_error() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService::create(&pool);

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let error = EmError::Generic(String::from("Executor 'post_error' test"));
        let error_message = error.to_string();
        executor_service.post_error(&executor_id, error).await;

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
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService::create(&pool);

        let executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        let mut listener = executor_service.status_listener(&executor_id).await?;

        sqlx::query(&format!("NOTIFY exec_status_{}, 'Test'", executor_id,))
            .execute(&pool)
            .await?;
        let update = listener.recv().await?;

        assert_eq!(ExecutorStatusUpdate::NoOp, update);

        Ok(())
    }

    #[sqlx::test]
    async fn clean_executors() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService::create(&pool);

        let inactive_executor_id = match executor_service.register_executor().await {
            Ok(inner) => inner,
            Err(error) => panic!("Failed to register a new executor, {}", error),
        };

        drop(executor_service);
        pool.close().await;
        drop(pool);

        let pool = PgConnectionBuilder::create_pool(db_options()?, 1, 1).await?;
        let executor_service = PgExecutorService::create(&pool);
        executor_service.clean_executors().await?;

        let status = executor_service.read_status(&inactive_executor_id).await?;

        assert_eq!(
            status,
            ExecutorStatus::Canceled,
            "Status was not Canceled for executor_id = {}",
            inactive_executor_id
        );

        Ok(())
    }
}

// use rocket::{
//     patch,
//     serde::{json::Json, msgpack::MsgPack},
//     State,
// };

// use super::utilities::{ApiFormatType, ApiResponse};
// use crate::services::task_queue::{TaskQueueRequest, PgTaskQueueService, TaskQueueService};

// /// Retry the task queue entry specified  by `request` regardless of the serialized format
// async fn task_queue_retry(
//     request: TaskQueueRequest,
//     service: &PgTaskQueueService,
//     format: ApiFormatType,
// ) -> ApiResponse<()> {
//     match service.retry_task(request).await {
//         Ok(_) => ApiResponse::message(
//             String::from("Successfully set task queue record to retry. Workflow scheduled for run"),
//             format,
//         ),
//         Err(error) => ApiResponse::error(error, format),
//     }
// }

// /// API endpoint to retry the task queue entry specified by `request`
// #[patch("/task-queue/retry?<f>", format = "json", data = "<request>")]
// pub async fn task_queue_retry_json(
//     request: Json<TaskQueueRequest>,
//     f: ApiFormatType,
//     service: &State<PgTaskQueueService>,
// ) -> ApiResponse<()> {
//     task_queue_retry(request.0, service, f).await
// }

// /// API endpoint to retry the task queue entry specified by `request`
// #[patch("/task-queue/retry?<f>", format = "msgpack", data = "<request>")]
// pub async fn task_queue_retry_msgpack(
//     request: MsgPack<TaskQueueRequest>,
//     f: ApiFormatType,
//     service: &State<PgTaskQueueService>,
// ) -> ApiResponse<()> {
//     task_queue_retry(request.0, service, f).await
// }

// /// Complete the task queue entry specified by `request` regardless of the serialized format
// async fn task_queue_complete(
//     request: TaskQueueRequest,
//     service: &PgTaskQueueService,
//     format: ApiFormatType,
// ) -> ApiResponse<()> {
//     match service.complete_task(request).await {
//         Ok(_) => ApiResponse::message(
//             String::from(
//                 "Successfully set task queue record to complete. Workflow scheduled for run",
//             ),
//             format,
//         ),
//         Err(error) => ApiResponse::error(error, format),
//     }
// }

// /// API endpoint complete the task queue entry specified by `request`
// #[patch("/task-queue/complete?<f>", format = "json", data = "<request>")]
// pub async fn task_queue_complete_json(
//     request: Json<TaskQueueRequest>,
//     f: ApiFormatType,
//     service: &State<PgTaskQueueService>,
// ) -> ApiResponse<()> {
//     task_queue_complete(request.0, service, f).await
// }

// /// API endpoint  complete the task queue entry specified by `request`
// #[patch("/task-queue/complete?<f>", format = "msgpack", data = "<request>")]
// pub async fn task_queue_complete_msgpack(
//     request: MsgPack<TaskQueueRequest>,
//     f: ApiFormatType,
//     service: &State<PgTaskQueueService>,
// ) -> ApiResponse<()> {
//     task_queue_complete(request.0, service, f).await
// }

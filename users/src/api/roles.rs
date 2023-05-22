use common::api::ApiResponse;
use uuid::uuid;

use crate::service::roles::{Role, RoleService};

/// API endpoint to fetch all roles
pub async fn roles<R>(service: actix_web::web::Data<R>) -> ApiResponse<Vec<Role>>
where
    R: RoleService,
{
    match service
        .read_all(&uuid!("be4c1ef7-771a-4580-b0dd-ff137c64ab48"))
        .await
    {
        Ok(roles) => ApiResponse::success(roles),
        Err(error) => ApiResponse::error(error),
    }
}

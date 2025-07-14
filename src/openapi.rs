use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::database::models::{
    AuthResponse, CreateUserRequest, ForgotPasswordRequest, LoginRequest, ResetPasswordRequest,
    UserInfo, UserRole,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ShiftLinkr API",
        version = "1.0.0",
        description = "A comprehensive workforce management API for scheduling, time-off, and employee management",
        contact(
            name = "ShiftLinkr Support",
            email = "support@shiftlinkr.com"
        )
    ),
    paths(
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::me,
        crate::handlers::auth::forgot_password,
        crate::handlers::auth::reset_password,
    ),
    components(
        schemas(
            AuthResponse,
            CreateUserRequest,
            ForgotPasswordRequest,
            LoginRequest,
            ResetPasswordRequest,
            UserInfo,
            UserRole,
            CompanyRole,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and account management"),
        (name = "Admin", description = "Administrative operations for locations, teams, and users"),
        (name = "Shifts", description = "Shift management and scheduling"),
        (name = "Time Off", description = "Time-off request management"),
        (name = "Swaps", description = "Shift swap request management"),
        (name = "Stats", description = "Dashboard and reporting statistics"),
        (name = "PTO Balance", description = "Paid time off balance management"),
        (name = "Companies", description = "Company and employee management"),
    ),
    security(
        ("Bearer" = [])
    )
)]
pub struct ApiDoc;

pub fn swagger_ui_service() -> SwaggerUi {
    SwaggerUi::new("/api/docs/{_:.*}")
        .url("/api/docs/openapi.json", ApiDoc::openapi())
}

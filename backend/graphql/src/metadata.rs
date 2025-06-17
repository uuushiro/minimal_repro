use actix_web_httpauth::extractors::bearer::BearerAuth;
use auth0_jwt_validator::{Auth0JwtValidator, Error};
use proto::Roles;

pub struct OrganizationId(pub i64);
pub struct Auth0Id(pub String);

pub struct UserMetadata {
    pub auth0_id: Auth0Id,
    pub organization_id: OrganizationId,
    pub roles: Roles,
    pub email: String,
}

pub async fn get_user_metadata(
    auth: BearerAuth,
    validator: actix_web::web::Data<Auth0JwtValidator>,
) -> Result<UserMetadata, actix_web::Error> {
    let token = auth.token();
    let claims = match validator.validate(token).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::error!("Failed to validate token: {:?}", e);
            if matches!(
                e,
                Error::InvalidToken
                    | Error::JWTDecode(_)
                    | Error::KIDNotFound
                    | Error::EmailNotVerified
            ) {
                return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
            } else {
                return Err(actix_web::error::ErrorInternalServerError(
                    "Internal server error",
                ));
            }
        }
    };
    let metadata = claims
        .custom_claims
        .and_then(|wrapper| wrapper.app_metadata)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization ID not found"))?;
    let organization_id = metadata.organization_id.unwrap_or_default();
    let auth0_id = claims.auth0_id;
    let roles = metadata
        .otooshi_roles
        .and_then(|otooshi_roles| Roles::from_base64(&otooshi_roles).ok())
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid token"))?;

    Ok(UserMetadata {
        organization_id: OrganizationId(organization_id),
        auth0_id: Auth0Id(auth0_id),
        roles,
        email: claims.email.unwrap_or_default(),
    })
}

use actix_web_httpauth::extractors::bearer::BearerAuth;
use auth0_jwt_validator::{Auth0JwtValidator, Error};
use proto::Roles;

pub(crate) async fn get_roles(
    auth: BearerAuth,
    validator: actix_web::web::Data<Auth0JwtValidator>,
) -> Result<Roles, actix_web::Error> {
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
    claims
        .custom_claims
        .and_then(|wrapper| wrapper.app_metadata)
        .and_then(|metadata| metadata.otooshi_roles)
        .and_then(|base64| Roles::from_base64(&base64).ok())
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid token"))
}

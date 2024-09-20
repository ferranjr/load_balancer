use std::fmt::Formatter;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::domain::urls::models::short_url::{CreateShortUrlRequest, RepositoryShortUrlError, ShortUrlId, ShortUrlResponse};
use crate::domain::urls::ports::{UrlsRepository, UrlsService};
use crate::domain::urls::service::Service;
use crate::inbound::http::handlers::short_urls::CreateShortUrlError::UnexpectedError;

#[derive(thiserror::Error)]
pub enum CreateShortUrlError {
    #[error("Malformed Url provided")]
    MalformedUrl,
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for CreateShortUrlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<RepositoryShortUrlError> for CreateShortUrlError {
    fn from(value: RepositoryShortUrlError) -> Self {
        match value {
            RepositoryShortUrlError::UrlBadlyFormatted { .. } => CreateShortUrlError::MalformedUrl,
            err => UnexpectedError(anyhow!(err))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateShortUrlResponse {
    short_url: Url,
    long_url: Url,
}

impl CreateShortUrlResponse {
    pub fn short_url(&self) -> &Url {
        &self.short_url
    }
    
    pub fn long_url(&self) -> &Url {
        &self.long_url
    }
}

impl From<ShortUrlResponse> for CreateShortUrlResponse {
    fn from(value: ShortUrlResponse) -> Self {
        Self {
            short_url: value.short_url().to_owned(),
            long_url: value.long_url().to_owned(),
        }
    }
}

impl ResponseError for CreateShortUrlError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateShortUrlError::MalformedUrl => StatusCode::BAD_REQUEST,
            UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub async fn create_short_url<R>(
    url_service: web::Data<Service<R>>,
    request: web::Json<CreateShortUrlRequest>,
) -> Result<HttpResponse, CreateShortUrlError> 
where 
    R: UrlsRepository
{
    let request = request.0;
    let short_url = url_service.create_short_url(&request).await?;
    let response = CreateShortUrlResponse::from(short_url);
    
    Ok(
        HttpResponse::Ok()
            .status(StatusCode::CREATED)
            .json(response)
    )
}

pub async fn get_short_url<R>(
    url_service: web::Data<Service<R>>,
    key: web::Path<ShortUrlId>,
) -> Result<HttpResponse, CreateShortUrlError>
where
    R: UrlsRepository
{
    let short_url = url_service.retrieve_short_url(key.to_owned()).await?;
    
    match short_url {
        None => Ok(
            HttpResponse::NotFound()
                .status(StatusCode::CREATED)
                .json({})
        ),
        Some(short_url_response) =>
            Ok(
                HttpResponse::SeeOther()
                .insert_header((LOCATION, short_url_response.long_url().as_str()))
                .finish()
            )
    }
    
}
use std::{error::Error, fmt::Display};

use log::{error, info};
use warp::{Reply, reject::custom};

#[derive(Debug)]
pub enum DbError {
	BoundaryViolation(String),
	DuplicateKey(String),
	EntityNotFound(String),
	ConstraintViolation(sqlx::Error),
	// NotTransacted(sqlx::Error),
	InvalidSql(sqlx::Error),
	NoConnection(sqlx::Error),
	UnknownError(sqlx::Error),
}

#[derive(Debug)]
pub enum ClientError {
	ParameterInvalid(String),
	ParameterMissing(String),
}


impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			DbError::BoundaryViolation(val) => write!(f, "Value boundary violated: {}", val),
			DbError::DuplicateKey(entity) => write!(f, "Duplicate key in entity {}", entity),
			DbError::EntityNotFound(typ) => write!(f, "Entity {} not found", typ),
			DbError::ConstraintViolation(e) => write!(f, "Constraint violated: {}", e),
			// DbError::NotTransacted => write!(f, "Not transacted"),
			DbError::InvalidSql(e) => write!(f, "Invalid SQL: {}", e),
			DbError::NoConnection(e) => write!(f, "Connection Error: {}", e),
			DbError::UnknownError(e) => write!(f, "{}", e),
		}
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			ClientError::ParameterInvalid(s) => write!(f, "Parameter value invalid: {}", s),
			ClientError::ParameterMissing(s) => write!(f, "Parameter {} is missing in the request", s),
		}
    }
}


impl Error for DbError {}

impl Error for ClientError {}


impl warp::reject::Reject for DbError {}
impl warp::reject::Reject for ClientError {}

impl From<DbError> for warp::Rejection {
	fn from(e: DbError) -> Self {
		custom(e)
	}
}

impl From<ClientError> for warp::Rejection {
	fn from(e: ClientError) -> Self {
		custom(e)
	}
}


impl From<sqlx::Error> for DbError {
	fn from(e: sqlx::Error) -> Self {
		match e {
			// sqlx::Error::Configuration(Box<dyn Error + 'static + Sync + Send, Global>),
			// sqlx::Error::Database(Box<dyn DatabaseError + 'static, Global>),
			// sqlx::Error::Io(Error),
			// sqlx::Error::Tls(Box<dyn Error + 'static + Sync + Send, Global>),
			// sqlx::Error::Protocol(String),
			sqlx::Error::RowNotFound => DbError::InvalidSql(e),
			sqlx::Error::ColumnIndexOutOfBounds {..} => DbError::InvalidSql(e),
			sqlx::Error::ColumnNotFound(_) => DbError::InvalidSql(e),
			sqlx::Error::ColumnDecode {..} => DbError::InvalidSql(e),
			sqlx::Error::Decode(_) => DbError::InvalidSql(e),
			sqlx::Error::PoolTimedOut => DbError::NoConnection(e),
			sqlx::Error::PoolClosed => DbError::NoConnection(e),
			// sqlx::Error::WorkerCrashed,
			// sqlx::Error::Migrate(Box<MigrateError, Global>),
			_ => DbError::UnknownError(e),
		}
	}
}

pub async fn handle_my_error(r: warp::Rejection) -> Result<impl Reply, warp::Rejection> {
	if let Some(e) = r.find::<DbError>() {
		info!("Request failed: {}", e);
		return match e {
			DbError::BoundaryViolation(_) => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
			DbError::DuplicateKey(_) => Ok(Box::new(warp::http::StatusCode::CONFLICT)),
			DbError::EntityNotFound(_) => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
			DbError::ConstraintViolation(_) => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
			DbError::InvalidSql(_) => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
			DbError::NoConnection(_) => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
			DbError::UnknownError(_) => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
			// _ => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
		};
	} else if let Some(e) = r.find::<ClientError>() {
		info!("Request failed: {}", e);
		return match e {
			ClientError::ParameterInvalid(_) => Ok(Box::new(warp::http::StatusCode::BAD_REQUEST)),
			ClientError::ParameterMissing(_) => Ok(Box::new(warp::http::StatusCode::BAD_REQUEST)),
		};
	} else {
		error!("Unknown error occurred.");
		return Err(r);
	}
}

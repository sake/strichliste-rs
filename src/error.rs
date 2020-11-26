use std::{error::Error, fmt::Display};

use warp::{Reply, reject::custom};

#[derive(Debug)]
pub enum DbError {
	EntityNotFound,
	ConstraintViolation,
	NotTransacted,
	InvalidSql,
	NoConnection,
	UnknownError,
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			DbError::EntityNotFound      => write!(f, "Entity not found"),
			DbError::ConstraintViolation => write!(f, "Constraint violated"),
			DbError::NotTransacted       => write!(f, "Not transacted"),
			DbError::InvalidSql          => write!(f, "Invalid SQL"),
			DbError::NoConnection        => write!(f, "No Connection"),
			DbError::UnknownError        => write!(f, "Unknown error"),
		}
    }
}

impl Error for DbError {

}

impl warp::reject::Reject for DbError {

}

impl From<DbError> for warp::Rejection {
	fn from(e: DbError) -> Self {
		custom(e)
	}
}

impl From<sqlx::Error> for DbError {
	fn from(e: sqlx::Error) -> Self {
		match e {
			// Configuration(Box<dyn Error + 'static + Sync + Send, Global>),
			// Database(Box<dyn DatabaseError + 'static, Global>),
			// Io(Error),
			// Tls(Box<dyn Error + 'static + Sync + Send, Global>),
			// Protocol(String),
			// RowNotFound,
			// ColumnIndexOutOfBounds {
			// 	index: usize,
			// 	len: usize,
			// },
			// ColumnNotFound(String),
			// ColumnDecode {
			// 	index: String,
			// 	source: Box<dyn Error + 'static + Sync + Send, Global>,
			// },
			// Decode(Box<dyn Error + 'static + Sync + Send, Global>),
			// PoolTimedOut,
			// PoolClosed,
			// WorkerCrashed,
			// Migrate(Box<MigrateError, Global>),
			_ => DbError::UnknownError,
		}
	}
}

pub async fn handle_my_error(r: warp::Rejection) -> Result<impl Reply, warp::Rejection> {
	if let Some(e) = r.find::<DbError>() {
		return match e {
			DbError::EntityNotFound => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
			DbError::UnknownError   => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
			_ => Ok(Box::new(warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
		};
	} else {
		return Err(r);
	}
}

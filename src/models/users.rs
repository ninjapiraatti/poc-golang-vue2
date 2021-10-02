use super::super::schema::*;
use crate::errors::ServiceError;
use crate::models;
use crate::storage::*;
use actix_identity::Identity;
use actix_web::{dev::Payload, web::Data, Error, FromRequest, HttpRequest};
use diesel::{r2d2::ConnectionManager, PgConnection};
use futures::future::{err, ok, Ready};
use log::debug;
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug, Insertable)]
#[table_name = "users"]
pub struct User {
	pub id: uuid::Uuid,
	pub isadmin: bool,
	pub email: String,
	pub username: String,
	pub hash: String,
	pub created_at: chrono::NaiveDateTime,
}

#[derive(Identifiable, Queryable, Serialize, Deserialize, Associations, PartialEq, Debug, Insertable)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "sessions"]
pub struct Session {
	pub id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	pub expire_at: chrono::NaiveDateTime,
	pub updated_by: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "activesessions"]
pub struct ActiveSession {
	pub session_id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	pub email: String,
	pub expire_at: chrono::NaiveDateTime,
	pub isadmin: bool,
}

impl User {
	pub fn from_details<S: Into<String>, T: Into<String>, U: Into<String>>(email: S, pwd: T, username: U) -> Self {
		let emailstr: String = email.into();
		User {
			id: uuid::Uuid::new_v4(),
			isadmin: false,
			email: String::from(&emailstr),
			username: username.into(),
			hash: pwd.into(),
			created_at: chrono::Local::now().naive_local(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggedUser {
	pub email: String,
	pub id: uuid::Uuid,
	pub session_id: uuid::Uuid,
	pub isadmin: bool,
}

impl From<ActiveSession> for LoggedUser {
	fn from(session: ActiveSession) -> Self {
		LoggedUser {
			email: session.email,
			id: session.user_id,
			session_id: session.session_id,
			isadmin: session.isadmin,
		}
	}
}

impl FromRequest for LoggedUser {
	type Config = ();
	type Error = Error;
	type Future = Ready<Result<LoggedUser, Error>>;

	fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
		if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
			if let Some(cookie) = identity.identity() {
				let pool = req.app_data::<Data<models::users::Pool>>().unwrap().clone();
				let id_res = uuid::Uuid::parse_str(&cookie);
				match id_res {
					Ok(id) => {
						let session = activesessions_storage::get_session_by_id(id, &pool);
						if let Ok(s) = session {
							if s.expire_at > chrono::offset::Utc::now().naive_utc() {
								let u: LoggedUser = s.into();
								return ok(u);
							}

							debug!("Session expired!");
						} else {
							debug!("No active session found!");
						}
					}
					Err(err) => {
						debug!("Not an UUID in the cookie! Error: {:?}", err);
					}
				};
			} else {
				debug!("Identity (cookie) not received!");
			}
		} else {
			debug!("Request processing failed!");
		}

		err(ServiceError::Unauthorized.into())
	}
}

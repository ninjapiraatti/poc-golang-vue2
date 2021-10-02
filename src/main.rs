#[macro_use]
extern crate diesel;

use actix_files as fs;
use aninmals;
//use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::Session;
use actix_web::http::{header, StatusCode};
use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::{error, info, trace};
//use diesel::r2d2::{self, ConnectionManager};

mod errors;
mod handlers;
mod models;
mod schema;
mod storage;
mod utils;
mod email_service;

#[get("/")]
async fn home(session: Session) -> Result<HttpResponse> {
	let mut counter = 1;
	if let Some(count) = session.get::<i32>("counter")? {
		counter = count + 1;
	}

	session.set("counter", counter)?;

	Ok(HttpResponse::build(StatusCode::OK)
		.content_type("text/html; charset=utf-8")
		.body(include_str!("../public/index.html")))
}

#[get("/app/*")]
async fn allviews(session: Session, req: HttpRequest) -> Result<HttpResponse> {
	println!("HTTP REQ:\n{:?}\n", req);
	let mut counter = 1;
	if let Some(count) = session.get::<i32>("counter")? {
		counter = count + 1;
	}

	session.set("counter", counter)?;

	Ok(HttpResponse::build(StatusCode::OK)
		.content_type("text/html; charset=utf-8")
		.body(include_str!("../public/index.html")))
}

fn initialize_db(name: &str) {
	info!("Running database migrations...");
	let connection = PgConnection::establish(&name).expect(&format!("Error connecting to {}", name));

	let result = diesel_migrations::run_pending_migrations(&connection);

	match result {
		Ok(_res) => {
			println!("Migrations done!");
		}
		Err(error) => {
			error!("Database migration error: \n {:#?}", error);
		}
	}
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	aninmals::public_function();
	dotenv::dotenv().ok();
	let rust_log = std::env::var("RUST_LOG").unwrap_or("info, simple-auth-server=debug".to_string());
	std::env::set_var("RUST_LOG", rust_log);
	env_logger::init();
	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	initialize_db(&database_url);
	let manager = ConnectionManager::<PgConnection>::new(database_url);
	let pool: models::users::Pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");
	let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
	let server_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "localhost:8086".to_string());
	println!("{:?}", domain);

	HttpServer::new(move || {
		App::new()
			.data(pool.clone())
			//.data(web::JsonConfig::default().limit(4096))
			.wrap(middleware::Logger::default())
			.service(
				web::scope("/api")
					.service(
						web::resource("/invitations")
							.route(web::post().to(handlers::invitation_handler::post_invitation)),
					)
					.service(
						web::resource("/test")
							.route(web::post().to(handlers::test_handler::test)),
					)
					.service(
						web::resource("/register/{invitation_id}")
							.route(web::post().to(handlers::register_handler::register_user)),
					)
			)
			.service(fs::Files::new("/public", "public").show_files_listing())
			.service(home)
			.service(allviews)
			.service(web::resource("/").route(web::get().to(|req: HttpRequest| {
				trace!("HTTP REQ:\n{:?}\n", req);
				HttpResponse::Found().header(header::LOCATION, "index.html").finish()
			})))
	})
	.bind(server_url)?
	.run()
	.await
}

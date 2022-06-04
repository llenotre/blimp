mod config;
mod dashboard;
mod global_data;
mod job;

use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::ContentType,
    middleware,
    web,
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder
};
use common::package::Package;
use common::package;
use common::request::PackageListResponse;
use common::request::PackageSizeResponse;
use common::version::Version;
use config::Config;
use global_data::GlobalData;
use serde_json::json;
use std::fs::File;
use std::sync::Mutex;

/// The server's version.
const VERSION: &str = "0.1";

#[get("/")]
async fn root() -> impl Responder {
    let body = format!("Blimp server version {}", VERSION);
    HttpResponse::Ok().body(body)
}

#[get("/motd")]
async fn motd(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
    let mut data = data.lock().unwrap();

    HttpResponse::Ok().body(data.get_config().motd.clone())
}

#[get("/package")]
async fn package_list(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
    let mut data = data.lock().unwrap();
    let packages = data.get_packages();

    match packages {
        Ok(packages) => HttpResponse::Ok().json(PackageListResponse {
			packages: packages.to_vec(),
		}),

        Err(e) => HttpResponse::InternalServerError().json(json!({
			"error": e.to_string(),
		})),
    }
}

#[get("/package/{name}/version/{version}")]
async fn package_info(
	req: HttpRequest,
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    // Getting package
    let package = Package::get(&name.to_owned(), &version).unwrap(); // TODO Handle error

    match package {
        Some(p) => HttpResponse::Ok().json(p),

        None => HttpResponse::NotFound().json(json!({
			"error": format!("Package `{}` with version `{}` not found", name, version),
		})),
    }
}

#[get("/package/{name}/version/{version}/size")]
async fn package_size(
	req: HttpRequest,
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    // Getting package
    let package = Package::get(&name.to_owned(), &version).unwrap(); // TODO Handle error

    match package {
        Some(_) => {
            let archive_path = format!("{}/{}-{}", package::SERVER_PACKAGES_DIR, name, version);
            let file = File::open(archive_path).unwrap(); // TODO Handle error
            let size = file.metadata().unwrap().len(); // TODO Handle error

            HttpResponse::Ok().json(PackageSizeResponse {
                size,
            })
        },

        None => {
            let json = json!({
                "error": format!("Package `{}` with version `{}` not found", name, version),
            });
            HttpResponse::NotFound().set(ContentType::json()).body(json)
        },
    }
}

#[get("/package/{name}/version/{version}/archive")]
async fn package_archive(
	req: HttpRequest,
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    let archive_path = format!("{}/{}-{}", package::SERVER_PACKAGES_DIR, name, version);
    NamedFile::open(archive_path) // TODO Make the error message cleaner
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // If the config doesn't exist, create it
    if !Config::exists() {
        Config::default().write().unwrap(); // TODO Handle error
    }

    // Reading config and initializing global data
    let config = Config::read().unwrap(); // TODO Handle error
    let port = config.port;

    let data = web::Data::new(Mutex::new(GlobalData::new(config)));

    // Enabling logging
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("[%t] %a: %r - Response: %s (in %D ms)"))
            .app_data(data.clone())
            .service(root)
            .service(motd)
            .service(package_list)
            .service(package_info)
            .service(package_size)
            .service(package_archive)
            .service(dashboard::home)
            .service(dashboard::style_css)
			.service(job::job_list)
			.service(job::job_get)
			.service(job::job_logs)
			.service(job::job_start)
			.service(job::job_stop)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}

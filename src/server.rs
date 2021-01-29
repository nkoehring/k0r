use super::db::{self, DBValue};
use super::render;
use super::templates::{self, statics::StaticFile};
use actix_web::{
    self,
    http::header::{ContentType, Expires, LOCATION},
    middleware::Logger,
    web, Error, HttpRequest, HttpResponse,
};
use std::time::{Duration, SystemTime};
use url::Url;

const CONTENT_TYPE_HTML: &str = "text/html; charset=utf-8";
const CONTENT_TYPE_JSON: &str = "application/json; charset=utf-8";

/// A duration to add to current time for a far expires header.
const FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

// TODO: make db::store_url aware of short codes that might lead to URLs
/// Common, unsoliticed queries by browsers that should be ignored
const IGNORED_SHORT_CODES: &[&str] = &["favicon.ico"];

type DB = web::Data<db::Pool>;
type JSON = web::Json<db::UrlPostData>;

fn get_request_origin(req: &HttpRequest) -> String {
    req.connection_info()
        .remote_addr()
        .unwrap_or("unkown origin")
        .to_string()
}

fn build_400_json(msg: &str) -> HttpResponse {
    let body = format!("{{\"status\": \"error\", \"message\": \"{}\"}}", msg);

    HttpResponse::BadRequest()
        .content_type(CONTENT_TYPE_JSON)
        .body(body)
}

fn build_404_json() -> HttpResponse {
    HttpResponse::BadRequest()
        .content_type(CONTENT_TYPE_JSON)
        .body("{{\"status\": \"error\", \"message\": \"Not Found\"}}")
}

fn build_500_json() -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type(CONTENT_TYPE_JSON)
        .body("{{\"status\": \"error\", \"message\": \"Internal Server Error\"}}")
}

/// Index page handler
#[actix_web::get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(CONTENT_TYPE_HTML)
        .body(render!(templates::index))
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
#[actix_web::get("/static/{filename}")]
fn static_file(path: web::Path<String>) -> HttpResponse {
    let name = &path.0;
    if let Some(data) = StaticFile::get(name) {
        let far_expires = SystemTime::now() + FAR;
        HttpResponse::Ok()
            .set(Expires(far_expires.into()))
            .set(ContentType(data.mime.clone()))
            .body(data.content)
    } else {
        HttpResponse::NotFound()
            .reason("No such static file.")
            .finish()
    }
}

/// Shortcode handler
/// Asks the database for the URL matching short_code and responds
/// with a redirect or, if not found, a JSON error
#[actix_web::get("/{short_code}")]
async fn redirect(req: HttpRequest, db: DB) -> Result<HttpResponse, Error> {
    let short_code = req.match_info().get("short_code").unwrap_or("0");

    if IGNORED_SHORT_CODES.contains(&short_code) {
        debug!(
            "{} queried {}: IGNORED",
            get_request_origin(&req),
            short_code
        );
        Ok(build_404_json())
    } else if let Ok(DBValue::String(url)) =
        db::query(&db, db::Queries::GetURL(short_code.to_owned())).await
    {
        debug!(
            "{} queried {}, got {}",
            get_request_origin(&req),
            &short_code,
            &url
        );
        Ok(HttpResponse::MovedPermanently()
            .header(LOCATION, url.clone())
            .content_type(CONTENT_TYPE_HTML)
            .body(render!(templates::redirect, "redirect", &url)))
    } else {
        debug!(
            "{} queried {}, got Not Found",
            get_request_origin(&req),
            short_code
        );
        Ok(build_404_json())
    }
}

#[actix_web::post("/")]
async fn add_url(_req: HttpRequest, data: JSON, db: DB) -> Result<HttpResponse, Error> {
    match Url::parse(&data.url) {
        Ok(parsed_url) => {
            if !parsed_url.has_authority() {
                debug!(
                    "{} posted \"{}\", got Invalid, no authority.",
                    get_request_origin(&_req),
                    &data.url
                );
                return Ok(build_400_json(
                    "Invalid URL, cannot be path only or data URL",
                ));
            }

            let query_result = db::query(&db, db::Queries::StoreNewURL(data.into_inner())).await;

            match query_result {
                Ok(DBValue::String(code)) => Ok(HttpResponse::Created()
                    .content_type(CONTENT_TYPE_JSON)
                    .body(format!("{{\"status\": \"ok\", \"message\": \"{}\"}}", code))),
                Err(err) => Ok(build_400_json(&format!("Invalid API key: {:?}", err))),
                _ => {
                    debug!(
                        "Got unexpected type back from StoreNewURL query: {:#?}",
                        query_result
                    );
                    Ok(build_500_json())
                }
            }
        }
        Err(_) => {
            debug!(
                "{} posted \"{}\", got Invalid, Parser Error.",
                get_request_origin(&_req),
                &data.url
            );
            Ok(build_400_json("Invalid URL"))
        }
    }
}

#[actix_web::main]
pub async fn start(db_pool: db::Pool) -> std::io::Result<()> {
    println!("Server is listening on 127.0.0.1:8080");

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(Logger::default())
            .data(db_pool.clone())
            .service(static_file) // GET /static/file.xyz
            .service(index) // GET /
            .service(redirect) // GET /123
            .service(add_url) // POST /
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

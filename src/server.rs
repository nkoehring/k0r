use std::path::PathBuf;
use r2d2_sqlite::SqliteConnectionManager;
use url::Url;
use actix_web::{self, App, HttpRequest, HttpResponse, HttpServer, Error as AWError};
use super::db;

const CONTENT_TYPE_HTML: &str = "content-type: text/html; charset=utf-8";
const CONTENT_TYPE_JSON: &str = "content-type: application/json; charset=utf-8";

type DB = actix_web::web::Data<db::Pool>;
type JSON = actix_web::web::Json<UrlPostData>;

#[actix_web::get("/")]
async fn index() -> HttpResponse {
    let body = "<h1>Welcome to k0r.eu</h1><p>This should be some explanatory page one day!</p>";

    HttpResponse::Ok()
    .content_type(CONTENT_TYPE_HTML)
    .body(body)
}

#[actix_web::get("/{short_code}")]
async fn redirect(req: HttpRequest, db: DB) -> Result<HttpResponse, AWError> {
    let respond_with_not_found = HttpResponse::NotFound()
        .content_type(CONTENT_TYPE_JSON)
        .body("{{\"status\": \"error\", \"message\": \"URL not found\"}}");

    let short_code = req.match_info().get("short_code").unwrap_or("0");

    #[cfg(feature = "debug-output")]
    let debug_info = format!(
        "{} queried \"{}\", got ",
        req.connection_info().remote_addr().unwrap_or("unkown origin"),
        short_code
    );

    if let Ok(url) = db::query(&db, db::Queries::GetURL(short_code.to_owned())).await {
        let body = format!("Would redirect to <a href=\"{}\">{}</a>.", url, url);

        #[cfg(feature = "debug-output")]
        println!("{}{}", debug_info, url);

        Ok(HttpResponse::Ok().content_type(CONTENT_TYPE_HTML).body(body))
    } else {
        #[cfg(feature = "debug-output")]
        println!("{}Not Found", debug_info);

        Ok(respond_with_not_found)
    }
}

#[derive(serde::Deserialize)]
struct UrlPostData {
    url: String,
}

#[actix_web::post("/")]
async fn add_url(_req: HttpRequest, data: JSON, db: DB) -> Result<HttpResponse, AWError> {
    let respond_with_bad_request = HttpResponse::BadRequest()
        .content_type("content-type: application/json; charset=utf-8")
        .body("{{\"status\": \"error\", \"message\": \"invalid URL\"}}");

    #[cfg(feature = "debug-output")]
    let debug_info = format!(
        "{} posted \"{}\", got ",
        _req.connection_info().remote_addr().unwrap_or("unkown origin"),
        &data.url
    );

    match Url::parse(&data.url) {
        Ok(parsed_url) => {
            if !parsed_url.has_authority() {
                #[cfg(feature = "debug-output")]
                println!("{}Invalid, no authority.", debug_info);

                return Ok(respond_with_bad_request);
            }
            let code = db::query(&db, db::Queries::StoreNewURL(data.url.clone())).await?;

            #[cfg(feature = "debug-output")]
            println!("{}{}.", debug_info, code);

            Ok(HttpResponse::Created()
                .content_type("content-type: application/json; charset=utf-8")
                .body(format!("{{\"status\": \"ok\", \"message\": \"{}\"}}", code)))
        },
        Err(_) => {
            #[cfg(feature = "debug-output")]
            println!("{}Invalid, Parser Error.", debug_info);

            Ok(respond_with_bad_request)
        },
    }
}

#[actix_web::main]
pub async fn start(db_path: PathBuf) -> std::io::Result<()> {
    #[cfg(feature = "debug-output")]
    println!("Using database {:?}", db_path.canonicalize().unwrap());

    let db_manager = SqliteConnectionManager::file(db_path);
    let db_pool = db::Pool::new(db_manager).unwrap();

    println!("Server is listening on 127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
        .data(db_pool.clone())
        .service(index)
        .service(redirect)
        .service(add_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

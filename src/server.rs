use actix_web::{self, web::{Data}, App, HttpRequest, HttpResponse, HttpServer};
use super::shortener::{Shortener, ShortenerResult, URLResult};

fn get_shortener() -> Shortener {
    if let ShortenerResult::Ok(shortener) = Shortener::from_file("test.urls") {
        #[cfg(feature = "debug-output")]
        println!("Found {} urls:\n{}", shortener.urls.len(), shortener.list_all());
        shortener
    } else {
        #[cfg(feature = "debug-output")]
        println!("Failed to load URL database. URL list will be empty.");
        Shortener::new(vec![])
    }
}

#[actix_web::get("/")]
async fn index() -> HttpResponse {
    let body = format!("<h1>Welcome to k0r.eu</h1><p>This should be some explanatory page one day!</p>");

    HttpResponse::Ok()
    .content_type("content-type: text/html; charset=utf-8")
    .body(body)
}

#[actix_web::get("/{short_code}")]
async fn redirect(req: HttpRequest, data: Data<Shortener>) -> HttpResponse {
    let short_code = req.match_info().get("short_code").unwrap_or("0");

    if let URLResult::Ok(url) = data.get_url(&short_code) {
        let body = format!("Would redirect to <a href=\"{}\">{}</a>.", url, url);
        HttpResponse::Ok()
        .content_type("content-type: text/html; charset=utf-8")
        .body(body)
    } else {
        HttpResponse::NotFound()
        .content_type("content-type: text/html; charset=utf-8")
        .body("<h1>404</h1><p>URL Not Found, sorry!</p>")
    }
}

#[actix_web::post("/{url}")]
async fn add_url(req: HttpRequest, data: Data<Shortener>) -> HttpResponse {
    let url = req.match_info().get("url").unwrap_or("invalid");

    if let Ok(short_code) = data.add_url(url) {
        HttpResponse::Created()
        .content_type("content-type: application/json; charset=utf-8")
        .body(format!("{{\"status\": \"ok\", \"message\": \"{}\"}}", short_code))
    } else {
        HttpResponse::BadRequest()
        .content_type("content-type: application/json; charset=utf-8")
        .body("{{\"status\": \"error\", \"message\": \"invalid URL\"}}")
    }
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    HttpServer::new(|| {
        let shortener = get_shortener();

        #[cfg(feature = "debug-output")]
        println!("Server is listening on 127.0.0.1:8080");

        App::new()
        .data(shortener)
        .service(index)
        .service(redirect)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

use std::fmt::{Display, Formatter, Result as FmtResult};

use actix_web::http::StatusCode;
use actix_web::{web::HttpResponse, ResponseError};
use serde::Serialize;
use serde_json::{json, to_string_pretty};

/// Error http response with the status code and a generic message.
/// Implements everything necessary to be consumed by actix-web.
#[derive(Debug, Serialize)]
pub struct Error {
    pub status: u16,
    pub msg: &'static str,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for Error {
    // builds the actix_web response
    fn error_response(&self) -> HttpResponse {
        let err_json = json!({ "error": self.msg });
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(err_json)
    }
}

impl Error {
    /// Returns a generic error with status code 400 and the given msg
    pub fn new(msg: &'static str) -> Error {
        Error { status: 400, msg }
    }

    /// Returns a generic not found error with status 404
    pub fn not_found() -> Error {
        Error {
            status: 404,
            msg: "Not Found",
        }
    }

    /// Returns a generic internal server error with status 500
    pub fn internal() -> Error {
        Error {
            status: 500,
            msg: "Internal Server Error",
        }
    }
}

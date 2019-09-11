use actix_web::{HttpResponse, ResponseError};
use std::fmt;
#[derive(Debug)]
pub enum MyError {
    NotAuthorized,
}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", "Client is not authorized")
    }
}
/// Actix web uses `ResponseError` for conversion of errors to a response
impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        match self {
            MyError::NotAuthorized => {
                println!("Client is not in our whitelist");
                HttpResponse::Unauthorized().finish()
            }
        }
    }
}

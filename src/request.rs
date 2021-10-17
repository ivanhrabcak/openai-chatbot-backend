use std::io::Cursor;

use rocket::{http::{ContentType, Status}, response::Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Response<T> where T: Send {
    pub response: T,
    pub status_code: u16
}

impl<'r, T> Responder<'r, 'static> for Response<T> where T: Serialize + Send {
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'static> {
        let json_string = match serde_json::to_string_pretty(&self) {
            Ok(result) => result,
            Err(_) => return Result::Err(Status::InternalServerError)
        };

        let response = rocket::Response::build()
            .sized_body(json_string.len(), Cursor::new(json_string))
            .header(ContentType::new("application", "json"))
            .status(Status::from_code(self.status_code).unwrap())
            .finalize();

        
        Result::Ok(response)
    }
}

impl<T> Response<T> where T: Send {
    pub fn new(response: T, status_code: u16) -> Self {
        Response { response, status_code }
    }
}
use http::{headers::ContentType, response::Writer, server::Server};

use crate::http::status::Status;

pub mod http;
fn main() {
    let mut srv = Server::new();

    let route1 = |response: &mut http::response::Response| {
        println!("that is numer 1 GET router");

        response
            .with_status(Status::OK)
            .headers()
            .set("base-".to_string(), "router-1".to_string());

        println!("{:?}", response.headers())
    };

    let route2 = |response: &mut http::response::Response| {
        println!("that is numer 2 GET router");

        response
            .headers()
            .set("base-".to_string(), "router-2".to_string());
        println!("{:?}", response.headers())
    };

    let route3 = |response: &mut http::response::Response| {
        response.headers().set(
            "Content-Type".to_string(),
            ContentType::ApplicationJson.to_string(),
        );

        let content = r#"
        {
            "method": "that is numer 3 POST router"
        }
        "#;

        if let Err(e) = response
            .with_status(Status::BadRequest)
            .write(content.to_string())
        {
            println!("{e}")
        };
    };

    srv.handler_get("/api/v1/route-number-1", Box::new(route1))
        .handler_get("/api/v1/route-number-2", Box::new(route2))
        .handler_post("/api/v1/route-number-3", Box::new(route3));

    srv.listen_and_serve("127.0.0.1:5437");
}

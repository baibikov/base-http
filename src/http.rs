pub mod headers {
    use std::collections::HashMap;

    pub enum ContentType {
        ApplicationJson,
    }

    impl ToString for ContentType {
        fn to_string(&self) -> String {
            match self {
                ContentType::ApplicationJson => "application/json".to_string(),
            }
        }
    }

    impl Default for ContentType {
        fn default() -> Self {
            Self::ApplicationJson
        }
    }

    #[derive(Debug)]
    pub struct Headers {
        headers: HashMap<String, String>,
    }

    impl ToString for Headers {
        fn to_string(&self) -> String {
            let mut ss: Vec<String> = Vec::new();
            for (k, v) in &self.headers {
                ss.push(format!("{k}: {v}"));
            }

            ss.join("\n")
        }
    }

    impl Headers {
        /// Creates a new [`Headers`].
        pub fn new() -> Self {
            Self {
                headers: HashMap::new(),
            }
        }

        /// set headers by http reposne
        pub fn set(&mut self, header_name: String, content_type: String) -> &mut Self {
            self.headers.insert(header_name, content_type);

            self
        }
    }

    impl Default for Headers {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub mod status {
    pub enum Status {
        OK = 200,
        BadRequest = 400,
        NotFound = 404,
        InternalServerError = 500,
    }

    impl Default for Status {
        fn default() -> Self {
            Self::OK
        }
    }

    impl ToString for Status {
        fn to_string(&self) -> String {
            match self {
                Status::OK => "OK".to_string(),
                Status::BadRequest => "Bad Request".to_string(),
                Status::NotFound => "Not Found".to_string(),
                Status::InternalServerError => "Internal Server".to_string(),
            }
        }
    }

    impl From<Status> for i32 {
        fn from(val: Status) -> Self {
            val as i32
        }
    }
}

pub mod server {
    use std::{
        collections::HashMap,
        io::{BufRead, BufReader},
        net::{TcpListener, TcpStream},
    };

    use crate::http::{headers::Headers, response::Response};

    pub type Handler = dyn Fn(&mut Response);

    pub struct Server {
        handlers: HashMap<(String, String), Box<Handler>>,
    }

    pub enum Method {
        GET,
        POST,
    }

    impl ToString for Method {
        fn to_string(&self) -> String {
            match self {
                Method::GET => "GET".to_string(),
                Method::POST => "POST".to_string(),
            }
        }
    }

    impl From<String> for Method {
        fn from(v: String) -> Self {
            if let "GET" = v.as_ref() {
                Method::GET
            } else {
                Method::POST
            }
        }
    }

    impl Server {
        pub fn new() -> Server {
            Server {
                handlers: HashMap::new(),
            }
        }

        pub fn handler_get(&mut self, pattern: &str, ffn: Box<Handler>) -> &mut Self {
            self.handlers
                .insert((Method::GET.to_string(), pattern.to_string()), ffn);

            self
        }

        pub fn handler_post(&mut self, pattern: &str, ffn: Box<Handler>) -> &mut Self {
            self.handlers
                .insert((Method::POST.to_string(), pattern.to_string()), ffn);

            self
        }

        pub fn listen_and_serve(&self, addr: &str) {
            let listener = TcpListener::bind(addr.to_string()).unwrap();

            for stream in listener.incoming() {
                self.accept_connection(stream.unwrap());
            }
        }

        fn accept_connection(&self, mut stream: TcpStream) {
            let buf_reader = BufReader::new(&mut stream);

            let http_request: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            let header_line: Vec<&str> = http_request.get(0).unwrap().split(" ").collect();

            let (method, route) = (header_line[0], header_line[1]);

            let handler = self.handlers.get(&(method.to_string(), route.to_string()));
            match handler {
                None => {
                    println!("error by unusable func")
                }
                Some(f) => {
                    let mut headers = Headers::new();
                    let mut resp = Response::from_stream_headers(stream, &mut headers);
                    f(&mut resp)
                }
            }

            println!("{:?}", http_request);
        }
    }
}

pub mod response {
    use std::io::{self, prelude::*};
    use std::net::TcpStream;

    use super::headers::Headers;
    use super::status::Status;

    pub struct Response<'a> {
        headers: &'a mut Headers,
        status: Status,
        stream: TcpStream,
    }

    impl<'a> Response<'a> {
        pub fn from_stream_headers(stream: TcpStream, headers: &'a mut Headers) -> Response<'a> {
            Response {
                headers,
                stream,
                status: Status::default(),
            }
        }

        pub fn headers(&mut self) -> &mut Headers {
            self.headers
        }
    }

    impl Response<'_> {
        pub fn with_status(&mut self, status: Status) -> &mut Self {
            self.status = status;
            self
        }
    }

    impl Writer<String> for Response<'_> {
        fn write(&mut self, buf: String) -> io::Result<()> {
            let headers = self.headers.to_string();
            let length = buf.len();
            let status_line = format!("HTTP/1.1 {} {}", self.status as i32, self.status.to_string());

            let content =
                format!("{status_line}\r\n{headers}\r\nContent-Length: {length}\r\n\r\n{buf}");

            self.stream.write_all(content.as_bytes())
        }
    }

    impl Writer<&[u8]> for Response<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<()> {
            self.stream.write_all(buf)
        }
    }

    pub trait Writer<T> {
        fn write(&mut self, buf: T) -> io::Result<()>;
    }
}

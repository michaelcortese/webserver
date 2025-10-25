use std::fs;
use std::thread;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

#[derive(Clone)]
enum HttpResponseType {
    Processing,
    Ok = 200,
    NotFound = 404,
}
struct HttpResponse {
    res_type: HttpResponseType,
    pub route: String,
}

impl HttpResponse {
    fn from(route: String) -> Self {
        Self {
            res_type: HttpResponseType::Processing,
            route,
        }
    }

    pub fn assemble(&mut self) -> String {
        self.res_type = HttpResponseType::Ok;
        let page = fs::read_to_string(&self.route).unwrap_or_else(|_| {
            {
                self.res_type = HttpResponseType::NotFound;
                fs::read_to_string("www/404.html")
            }
            .unwrap_or(String::from("Error: not found"))
        });

        match self.res_type {
            HttpResponseType::NotFound => format!(
                "HTTP/1.1 404 Not Found\r\n\
             Content-Length: {}\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Connection: close\r\n\
             \r\n{}",
                &page.len(),
                page
            ),
            HttpResponseType::Ok => format!(
                "HTTP/1.1 200 OK\r\n\
             Content-Length: {}\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Connection: close\r\n\
             \r\n{}",
                &page.len(),
                page
            ),
            _ => format!(
                "HTTP/1.1 500 Internal Server Error\r\n\
             Content-Length: {}\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Connection: close\r\n\
             \r\n{}",
                &page.len(),
                page
            ),
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(("::", 8080))?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut connection) => {
                thread::spawn(move || {
                    if let Err(e) = handle_connection(&mut connection) {
                        eprintln!("Err {}", e)
                    }
                });
            }

            Err(err) => {
                panic!("{err}")
            }
        }
    }
    Ok(())
}

fn handle_connection(connection: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    connection.read_exact(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer);

    let mut res = request.lines().next().unwrap_or("").split_whitespace();

    if let Some(http) = res.next() {
        match http {
            "GET" => {
                let mut http_res = HttpResponse::from({
                    let mut route = res.next().unwrap_or("404.html");
                    route = route.trim_start_matches("/");
                    if route.is_empty() {
                        route = "index.html"
                    }
                    format!("www/{}", route)
                });
                connection.write_all(http_res.assemble().as_bytes())?;
                connection.flush()?;
                connection.shutdown(Shutdown::Both)?;
            }
            _ => {
                let mut http_res = HttpResponse::from(format!(
                    "www/{}",
                    res.next().unwrap().trim_start_matches("/")
                ));
                connection.write_all(http_res.assemble().as_bytes())?;
                connection.flush()?;
                connection.shutdown(Shutdown::Both)?;
            }
        }
    }

    // conclude request

    Ok(())
}

extern crate hello;
use hello::ThreadPool;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("localhost:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        pool.execute(|| {
            handle_connection(stream.unwrap());
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    if method(&buffer) == "GET" {
        let path = path(&buffer);

        if path == "/sleep" {
            thread::sleep(Duration::from_secs(5));
        }

        if let Ok(file) = File::open(&format!("static{}", path)) {
            respond("200 OK", file, stream);
            return;
        }
    }
    respond(
        "404 NOT FOUND",
        File::open("static/404.html").unwrap(),
        stream,
    );
}

fn method(req_buffer: &[u8]) -> String {
    let method = req_buffer
        .iter()
        .take_while(|c| **c != b' ')
        .map(|c| *c)
        .collect::<Vec<_>>();

    String::from_utf8_lossy(&method).into_owned()
}

fn path(req_buffer: &[u8]) -> String {
    let path = req_buffer
        .iter()
        .skip_while(|c| **c != b' ')
        .skip(1)
        .take_while(|c| **c != b' ')
        .map(|c| *c)
        .collect::<Vec<_>>();

    let mut path = String::from_utf8_lossy(&path).into_owned();

    if path.chars().last().unwrap() == '/' {
        path.push_str("index.html");
    }

    path
}

fn respond(status: &str, mut file: File, mut stream: TcpStream) {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("HTTP/1.1 {}\r\n\r\n{}", status, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_extracts_method() {
        assert_eq!(method(b"GET / HTTP/1.1"), "GET");
        assert_eq!(method(b"POST /hello.html HTTP/1.1"), "POST");
        assert_eq!(method(b"DELETE /foo/ HTTP/1.1"), "DELETE");
    }

    #[test]
    fn path_extracts_path() {
        assert_eq!(path(b"GET / HTTP/1.1"), "/index.html");
        assert_eq!(
            path(b"POST /hello.html HTTP/1.1"),
            "/hello.html"
        );
        assert_eq!(
            path(b"DELETE /foo/ HTTP/1.1"),
            "/foo/index.html"
        );
    }
}

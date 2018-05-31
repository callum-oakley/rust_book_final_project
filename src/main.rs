use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("localhost:7878").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    if method(&buffer[..]) == "GET" {
        let filename = format!("static{}", path(&buffer[..]));
        if let Ok(file) = File::open(&filename) {
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

fn method(req: &[u8]) -> String {
    let mut i = 0;
    for &c in req {
        if c == b' ' {
            break;
        }
        i += 1
    }
    String::from_utf8_lossy(&req[..i]).into_owned()
}

fn path(req: &[u8]) -> String {
    let mut i = 0;
    loop {
        if req[i] == b' ' {
            break;
        }
        i += 1;
    }
    i += 1;

    let mut j = i;
    loop {
        if req[j] == b' ' {
            break;
        }
        j += 1;
    }

    let path = String::from_utf8_lossy(&req[i..j]).into_owned();

    if req[j - 1] == b'/' {
        format!("{}{}", path, "index.html")
    } else {
        path
    }
}

fn respond(status: &str, mut file: File, mut stream: TcpStream) {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("HTTP/1.1 {}\r\n\r\n{}", status, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

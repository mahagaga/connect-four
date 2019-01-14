extern crate server;
extern crate iron;
extern crate hyper;
extern crate regex;

use server::start_server;
use iron::Listening;
use std::io::Read;
use regex::Regex;

#[test]
fn it_works() {
    let server = TestServer::new();
    let client = hyper::Client::new();

    fn check_response(q:&str, a:&str, s:&TestServer, c: &hyper::Client) {
        let url = format!("{}/{}", s.url(), q);
        let mut response = c.get(&url).send().unwrap();
        let mut s = String::new();
        response.read_to_string(&mut s).unwrap();

        let expectation = Regex::new(a).unwrap();
        println!("{}", s);
        assert!(expectation.is_match(s.as_str()));
    }

    for (question, answer) in vec![
        ("new", "[{] \"field\": \"-{6}([\\\\]n){8}-{6}\", \"gameid\": [0-9]+ [}]"),
        ("new", "[{] \"field\": \"-{6}([\\\\]n){8}-{6}\", \"gameid\": [0-9]+ [}]"),
//        ("move/?/black/4", "[{] \"field\": \"-{6}([\\\\]n){5}o([\\\\]n){3}-{6}\" [}]"),
//        ("move/?/black/5", "[{] \"field\": \"-{6}([\\\\]n){6}x([\\\\]n){5}-{6}\" [}]"),
    ] {
        check_response(question, answer, &server, &client)
    }
}

struct TestServer(Listening);

impl TestServer {
    fn new() -> TestServer {
        TestServer(start_server("127.0.0.1", 0))
    }

    fn url(&self) -> String {
        format!("http://{}:{}", self.0.socket.ip(), self.0.socket.port())
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.0.close().expect("Error closing server");
    }
}

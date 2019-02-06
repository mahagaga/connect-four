extern crate server;
extern crate game;
extern crate iron;
extern crate hyper;
extern crate regex;

use server::start_server;
use game::connectfour::ConnectFourStrategy;
use iron::Listening;
use std::io::Read;
use regex::Regex;

#[test]
fn it_works() {
    let server = TestServer::new();
    let client = hyper::Client::new();

    fn check_response<'a>(q:&str, a:&str, s:&TestServer, c: &hyper::Client) -> Vec<String>  {
        let url = format!("{}/{}", s.url(), q);
        let mut response = c.get(&url).send().unwrap();
        let mut rs = String::new();
        response.read_to_string(&mut rs).unwrap();

        let expectation = Regex::new(a).unwrap();
        println!("{} -> {}", q, rs);
        assert!(expectation.is_match(rs.as_str()));
        
        let caps = expectation.captures(rs.as_str()).unwrap();
        caps.iter().map(|m| String::from(m.unwrap().as_str())).collect()
    }

    let gameid1 = check_response("new", "[{] \"field\": \"-{6}([\\\\]n){8}-{6}\", \"gameid\": ([0-9]+) [}]", &server, &client).pop().unwrap();
    let gameid2 = check_response("new", "[{] \"field\": \"-{6}([\\\\]n){8}-{6}\", \"gameid\": ([0-9]+) [}]", &server, &client).pop().unwrap();
    check_response(format!("move/{}/white/4", gameid1).as_str(), "[{] \"field\": \"-{6}([\\\\]n){5}o([\\\\]n){3}-{6}\" [}]", &server, &client);
    check_response(format!("move/{}/black/5", gameid2).as_str(), "[{] \"field\": \"-{6}([\\\\]n){6}x([\\\\]n){2}-{6}\" [}]", &server, &client);
}

struct TestServer(Listening);

impl TestServer {
    fn new() -> TestServer {
        TestServer(start_server("127.0.0.1", 0, ConnectFourStrategy::default()))
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

extern crate server;
extern crate iron;
extern crate hyper;

use server::start_server;
use iron::Listening;
use std::io::Read;

#[test]
fn it_works() {
    let server = TestServer::new();
    let client = hyper::Client::new();

    fn check_response(q:&str, a:&str, s:&TestServer, c: &hyper::Client) {
        let url = format!("{}/{}", s.url(), q);
        let mut response = c.get(&url).send().unwrap();
        let mut s = String::new();
        response.read_to_string(&mut s).unwrap();

        assert_eq!(a, s);
    }

    for (question, answer) in vec![
        ("new", "{ \"field\": \"------\\n\\n\\n\\n\\n\\n\\n\\n------\", \"gameid\": 1 }"),
        ("new", "{ \"field\": \"------\\n\\n\\n\\n\\n\\n\\n\\n------\", \"gameid\": 2 }"),
        ("move/1/black/4", "{ \"field\": \"------\\n\\n\\n\\n\\no\\n\\n\\n------\" }"),
        ("move/2/white/5", "{ \"field\": \"------\\n\\n\\n\\n\\n\\nx\\n\\n------\" }"),
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

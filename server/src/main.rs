extern crate server;
extern crate game;

fn main() {
    let strategy = game::connectfour::ConnectFourStrategy::default();
    server::start_server("localhost", 8095, strategy);
}

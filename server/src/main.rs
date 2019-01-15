extern crate server;
extern crate game;

fn main() {
    let strategy = game::ConnectFourStrategy::default();
    server::start_server("localhost", 8095, strategy);
}

extern crate game;
use game::*;

extern crate iron;

use iron::prelude::*;
use iron::status;

use iron::Handler;

struct ConnectFourHandler {
    cf: ConnectFour,
    st: ConnectFourStrategy,
}

impl Handler for ConnectFourHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        fn readurl(req: &Request) -> (Option<Player>, Option<Column>) {
            let mut player = None;
            if let Some(p) = &req.url.path().get(1) {
                player = match **p {
                    "black" => Some(Player::Black),
                    "white" => Some(Player::White),
                    _ => None,
                };
            }
            let mut column = None;
            if let Some(c) = &req.url.path().get(2) {
                column = match **c {
                    "0" => Some(Column::One),
                    "1" => Some(Column::Two),
                    "2" => Some(Column::Three),
                    "3" => Some(Column::Four),
                    "4" => Some(Column::Five),
                    "5" => Some(Column::Six),
                    "6" => Some(Column::Seven),
                    _ => None,
                };
            }
            (player, column)
        }
        
        if let Some(s) = &req.url.path().get(0) {
            let mut answer = String::new();
            match **s {
                "new" => { 
                    self.cf = ConnectFour::new(); 
                    answer = self.cf.display(); 
                },
                "move" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        answer = self.cf.display();
                    } else { return Ok(Response::with(status::BadRequest)) }
                },
                "withdraw" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        //answer = self.cf.display();
                    } else { return Ok(Response::with(status::BadRequest)) }
                },
                "eval" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        //answer = self.st.eval();
                    } else { return Ok(Response::with(status::BadRequest)) }
                },
                "best" => {
                    if let (Some(player), _) = readurl(&req) {
                        //answer = self.st.find_best_move();
                    } else { return Ok(Response::with(status::BadRequest)) }                    
                },
                _ => return Ok(Response::with(status::BadRequest)),
            }
            Ok(Response::with((status::Ok, answer.as_str())))
        } else { return Ok(Response::with(status::BadRequest)) }
    }
}

fn main() {
    let mut cf = ConnectFour::new();

    fn connect_four(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    let _server = Iron::new(ConnectFourHandler {
        cf: ConnectFour::new(),
        st: ConnectFourStrategy { 
            mscore_koeff: 1.0,
            oscore_koeff: 0.8,
            nscore_koeff: 0.5,
        }
    }).http("localhost:8095").unwrap();
    println!("On 8095");
}

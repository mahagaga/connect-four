extern crate game;
use game::*;

extern crate iron;

use iron::prelude::*;
use iron::status;

use iron::Handler;

use std::sync::Mutex;
use std::rc::Rc;
use std::cell::RefCell;

struct ConnectFourHandler {
    cf: Mutex<ConnectFour>,
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
        
        let mut answer = None;

        if let Some(s) = &req.url.path().get(0) {
            let mut cf = self.cf.lock().unwrap();

            match **s {
                "new" => { 
                    *cf = ConnectFour::new(); 
                    answer = Some(cf.display()); 
                },
                "move" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        if let Ok(_) = cf.drop_stone(&player, column) {
                            answer = Some(cf.display()); 
                        }
                    }
                },
                "withdraw" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        if let Ok(_) = cf.make_move(&player, Rc::new(ConnectFourMove{ data: column, })) {
                            answer = Some(cf.display()); 
                        }
                    }
                },
                "eval" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        let cfclone = cf.clone();
                        if let Ok(eval) = self.st.evaluate_move(Rc::new(RefCell::new(cfclone)), &player, Rc::new(ConnectFourMove{ data: column, })) {
                            answer = Some(format!("{}", eval));
                        }
                    }
                },
                "best" => {
                    if let (Some(player), _) = readurl(&req) {
                        let cfclone = cf.clone();
                        if let (Some(mv), Some(_score)) = self.st.find_best_move(Rc::new(RefCell::new(cfclone)), &player, 4, true) {
                            if let Ok(_) = cf.make_move(&player, mv) {
                                answer = Some(cf.display());
                            }
                        }
                    }                    
                },
                _ => (),
            }
        }
        if let Some(line) = answer {
            Ok(Response::with((status::Ok, line.as_str())))
        } else { return Ok(Response::with(status::BadRequest)) }
    }
}

fn main() {

    let _server = Iron::new(ConnectFourHandler {
        cf: Mutex::new(ConnectFour::new()),
        st: ConnectFourStrategy { 
            mscore_koeff: 1.0,
            oscore_koeff: 0.8,
            nscore_koeff: 0.5,
        }
    }).http("localhost:8095").unwrap();
    println!("On 8095");
}

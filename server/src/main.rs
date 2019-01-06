extern crate game;
use game::*;

extern crate iron;
extern crate hyper;

use iron::prelude::*;
use iron::status;

use iron::Handler;

use std::sync::Mutex;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

struct ConnectFourHandler {
    cfm: Mutex<HashMap<i32,ConnectFour>>,
    st: ConnectFourStrategy,
}

use hyper::header::AccessControlAllowOrigin;

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

        let mut evaluation_clone:Option<ConnectFour> = None;
        let mut best_move_clone:Option<ConnectFour> = None;

        if let Some(s) = &req.url.path().get(0) {

            let mut cfm = self.cfm.lock().unwrap();

            match **s {
                "new" => {
                    (*cfm).insert(1, ConnectFour::new());
                    // answer must be proper JSON (", no ', \\n, no \n) for ajax
                    answer = Some(format!("{{ \"field\": \"{}\" }}", (*cfm).get(&1).unwrap().display().replace("\n", "\\n")));
                },
                "move" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        let mut cfg = (*cfm).remove(&1).unwrap();
                        if let Ok(_) = cfg.drop_stone(&player, column) {
                            answer = Some(format!("{{ \"field\": \"{}\" }}", cfg.display().replace("\n", "\\n")));
                        }
                        (*cfm).insert(1, cfg);
                    }
                },
                "withdraw" => {
                    if let (Some(player), Some(column)) = readurl(&req) {
                        let mut cfg = (*cfm).remove(&1).unwrap();
                        cfg.withdraw_move(&player, Rc::new(ConnectFourMove{ data: column, }));
                        answer = Some(format!("{{ \"field\": \"{}\" }}", cfg.display().replace("\n", "\\n")));
                        (*cfm).insert(1, cfg);
                    }
                },
                "eval" => {
                    evaluation_clone = Some((*cfm).get(&1).unwrap().clone());
                },
                "best" => {
                    best_move_clone = Some((*cfm).get(&1).unwrap().clone());
                },
                _ => (),
            }
        }

        // by now the lock on cfc is released, so the expensive calculations below do not inhibit other threads
        if let Some(cfclone) = evaluation_clone {
            if let (Some(player), Some(column)) = readurl(&req) {
                if let Ok(eval) = self.st.evaluate_move(Rc::new(RefCell::new(cfclone)), &player, Rc::new(ConnectFourMove{ data: column, })) {
                    answer = Some(format!("{{ \"evaluation\": {} }}", eval));
                }
            }
        }
        if let Some(cfclone) = best_move_clone {
            if let (Some(player), _) = readurl(&req) {
                if let (Some(mv), Some(_score)) = self.st.find_best_move(Rc::new(RefCell::new(cfclone)), &player, 4, true) {
                    answer = Some(format!("{{ \"bestmove\": {} }}", mv.data().to_usize()));
                }
            }                    
        }

        if let Some(line) = answer {
            let mut response = Response::with((status::Ok, line.as_str()));
            // allow all origins, so the service can be called from javascript
            response.headers.set(AccessControlAllowOrigin::Any);
            Ok(response)
        } else { return Ok(Response::with(status::BadRequest)) }
    }
}

fn main() {

    let _server = Iron::new(ConnectFourHandler {
        cfm: Mutex::new(HashMap::new()),
        st: ConnectFourStrategy { 
            mscore_koeff: 1.0,
            oscore_koeff: 0.8,
            nscore_koeff: 0.5,
        }
    }).http("localhost:8095").unwrap();
    println!("On 8095");
}

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
use std::time::{Instant, Duration};
use std::thread::sleep;

struct ConnectFourHandler {
    zero: Instant,
    cfm: Mutex<HashMap<u128,ConnectFour>>,
    st: ConnectFourStrategy,
}

use hyper::header::AccessControlAllowOrigin;

impl Handler for ConnectFourHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        fn readurl(req: &Request) -> (Option<u128>, Option<Player>, Option<Column>) {
            let mut gameid = None;
            if let Some(id) = &req.url.path().get(1) {
                if let Ok(g) = (**id).parse::<u128>() {
                    gameid = Some(g);
                }
            }
            let mut player = None;
            if let Some(p) = &req.url.path().get(2) {
                player = match **p {
                    "black" => Some(Player::Black),
                    "white" => Some(Player::White),
                    _ => None,
                };
            }
            let mut column = None;
            if let Some(c) = &req.url.path().get(3) {
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
            (gameid, player, column)
        }
        
        fn key_from_time(map: &HashMap<u128,ConnectFour>, then: Instant) -> u128 {
            let mut now = Instant::now();
            let mut key: u128 = now.duration_since(then).as_secs() as u128 * 1000 + now.duration_since(then).subsec_millis() as u128;
            while let Some(_) = map.get(&key) {
                sleep(Duration::from_millis(1));
                now = Instant::now();
                key = now.duration_since(then).as_secs() as u128 * 1000 + now.duration_since(then).subsec_millis() as u128;
            }
            key
        }

        let mut answer = None;

        let mut evaluation_clone:Option<ConnectFour> = None;
        let mut best_move_clone:Option<ConnectFour> = None;

        if let Some(s) = &req.url.path().get(0) {

            let mut cfm = self.cfm.lock().unwrap();

            match **s {
                "new" => {
                    let key = key_from_time(&(*cfm), self.zero);
                    (*cfm).insert(key, ConnectFour::new());
                    // answer must be proper JSON (", no ', \\n, no \n) for ajax
                    answer = Some(format!("{{ \"field\": \"{}\", \"gameid\": {} }}", (*cfm).get(&1).unwrap().display().replace("\n", "\\n"), key));
                },
                "move" => {
                    if let (Some(gameid), Some(player), Some(column)) = readurl(&req) {
                        // move game out of map ...
                        let mut cfg = (*cfm).remove(&gameid).unwrap();
                        if let Ok(_) = cfg.drop_stone(&player, column) {
                            answer = Some(format!("{{ \"field\": \"{}\" }}", cfg.display().replace("\n", "\\n")));
                        }
                        // ... and in again for gaining ownership
                        (*cfm).insert(1, cfg);
                    }
                },
                "withdraw" => {
                    if let (Some(gameid), Some(player), Some(column)) = readurl(&req) {
                        let mut cfg = (*cfm).remove(&gameid).unwrap();
                        cfg.withdraw_move(&player, Rc::new(ConnectFourMove{ data: column, }));
                        answer = Some(format!("{{ \"field\": \"{}\" }}", cfg.display().replace("\n", "\\n")));
                        (*cfm).insert(gameid, cfg);
                    }
                },
                "eval" => {
                    if let Some(id) = &req.url.path().get(1) {
                        if let Ok(gameid) = (**id).parse::<u128>() {
                            evaluation_clone = Some((*cfm).get(&gameid).unwrap().clone());
                        }
                    }   
                },
                "best" => {
                    if let Some(id) = &req.url.path().get(1) {
                        if let Ok(gameid) = (**id).parse::<u128>() {
                            best_move_clone = Some((*cfm).get(&gameid).unwrap().clone());
                        }
                    }   
                },
                _ => (),
            }
        }

        // by now the lock on cfc is released, so the possibly expensive calculations below do not inhibit other threads
        if let Some(cfclone) = evaluation_clone {
            if let (Some(_), Some(player), Some(column)) = readurl(&req) {
                if let Ok(eval) = self.st.evaluate_move(Rc::new(RefCell::new(cfclone)), &player, Rc::new(ConnectFourMove{ data: column, })) {
                    answer = Some(format!("{{ \"evaluation\": {} }}", eval));
                }
            }
        }
        if let Some(cfclone) = best_move_clone {
            if let (Some(_), Some(player), _) = readurl(&req) {
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

pub fn start_server(host:&str, port:i32) -> iron::Listening {
    let server = Iron::new(ConnectFourHandler {
        zero: Instant::now(),
        cfm: Mutex::new(HashMap::new()),
        st: ConnectFourStrategy { 
            mscore_koeff: 1.0,
            oscore_koeff: 0.8,
            nscore_koeff: 0.5,
        }
    }).http(format!("{}:{}", host, port)).unwrap();
    println!("On {}", port);
    server
}

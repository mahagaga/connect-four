//pub mod generic;
use generic::{Game, Move, Player, Score, Strategy, Withdraw};
use connectfour::{Column, ConnectFour};
use std::rc::Rc;
use std::cell::RefCell;


pub struct BruteForceStrategy {
    workers:Vec<Worker>,
    control:Controller,
}

enum Eval {
    Available(Score),
    InCalculation,
}

use std::collections::HashMap;
struct Store {
    games:HashMap<String,ConnectFour>,
    scores:HashMap<String,Eval>,
    calrec:HashMap<String,String>,
}

impl Store {
    fn new() -> Self {
        Store {
            games: HashMap::new(),
            scores: HashMap::new(),
            calrec: HashMap::new(),
        }
    }
}

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

struct Worker {
    out: Sender<String>,
    een: Receiver<String>,
}

impl Worker {
    fn parse_request(request:String) -> (ConnectFour,Player) {
        (ConnectFour::new(), Player::White)
    }
    fn get_it_done(&self, request: String) -> String {
        let (mut game, player) = Worker::parse_request(request);
        game.possible_moves(&player).iter()
        .map(|mv|{
            game.make_move(&player, mv.clone());
        });
        String::from("")
    }

    fn new(game: ConnectFour, player: Player) -> Self {
        let (sender, receiver) = channel();
        Worker {
            out: sender,
            een: receiver,
        }
    }

    fn run(self) {
        let handle = thread::spawn(move || {
            while true {
                let request:String = self.een.recv().unwrap();
                let answer = self.get_it_done(request);
                self.out.send(answer).unwrap()
            };
        });
    }
}
struct Controller {
    store: Store,
}

impl Controller {
    fn new() -> Self {
        Controller { store: Store::new(), }
    }
}

impl BruteForceStrategy {
    pub fn new(nworker:i32) -> Self {
        BruteForceStrategy { 
            workers: Vec::new(),
            control: Controller::new(),
        }
    }

    pub fn pave_ground(&self, 
            g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
            p: &Player, toplimit: i32) {
    }
}

impl Strategy<Column,Vec<Vec<Option<Player>>>> for BruteForceStrategy {
    
    fn evaluate_move(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                     p: &Player, mv: Rc<Move<Column>>) 
    -> Result<f32, Withdraw> {
        Ok(0.0)
    }
}
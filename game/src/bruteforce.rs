//pub mod generic;
use generic::{Game, Move, Player, Score, Strategy, Withdraw};
use connectfour::{Column, ConnectFour};
use std::rc::Rc;
use std::cell::RefCell;


pub struct BruteForceStrategy {
    workers:Vec<Worker>,
    control:Controller,
    query_receiver: Receiver<QueryM>,
    record_sender: Vec<Sender<StoreM>>,
    done_receiver: Receiver<DoneM>,
    job_sender: Vec<Sender<JobM>>,
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

struct Hash {
    hash: String,
}

enum ScoreEntry {
    Won,
    Lost,
    Draw,
    InCalculation,
    Unknown,
    Missing,
}

struct QueryM {
    worker_id: i32,
    hash: Hash,
    score: ScoreEntry,
}

struct StoreM {
    hash: Hash,
    score: ScoreEntry,
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
    worker_id: i32,
    query: Sender<QueryM>,
    record: Receiver<StoreM>,
    ctl_in: Receiver<JobM>,
    ctl_out: Sender<DoneM>,
}

fn hash(game:&ConnectFour) -> Hash {
    Hash{hash:String::from(""),}
}

impl Worker {
    fn parse_request(request:JobM) -> (ConnectFour,Player) {
        (ConnectFour::new(), Player::White)
    }

    fn get_entry(&self, hash:Hash) -> ScoreEntry {
        ScoreEntry::InCalculation
    }

    fn treat_undecided(&self, undecided: Vec<std::rc::Rc<dyn Move<Column>>>, mut game: ConnectFour, player:&Player) {
        let mut wins = Vec::new();
        let mut draws = Vec::new();
        let mut losses = Vec::new();
        let mut open = Vec::new();
        let mut incalculation = Vec::new();

        for mv in undecided.iter() {
            game.make_move(player, mv.clone()).unwrap();
            match self.get_entry(hash(&game)) {
                ScoreEntry::Won => wins.push(mv.clone()),
                ScoreEntry::Lost => losses.push(mv.clone()),
                ScoreEntry::Draw => draws.push(mv.clone()),
                ScoreEntry::InCalculation => incalculation.push(mv.clone()),
                ScoreEntry::Unknown => open.push(mv.clone()),
                ScoreEntry::Missing => open.push(mv.clone()),
            }
            game.withdraw_move(&player, mv.clone());
        }

        while let Some(mv) = incalculation.pop() {
            game.make_move(player, mv.clone()).unwrap();
            match self.get_entry(hash(&game)) {
                ScoreEntry::Won => wins.push(mv.clone()),
                ScoreEntry::Lost => losses.push(mv.clone()),
                ScoreEntry::Draw => draws.push(mv.clone()),
                ScoreEntry::InCalculation => incalculation.insert(0, mv.clone()),
                ScoreEntry::Unknown => open.push(mv.clone()),
                ScoreEntry::Missing => open.push(mv.clone()),
            }
            game.withdraw_move(&player, mv.clone());
        }

        if wins.len()>0 {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Won, hash: hash(&game), }).unwrap();
        } else if open.len()>0 {
            




        } else if draws.len()>0 {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Draw, hash: hash(&game), }).unwrap();
        } else if losses.len()>0 {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Lost, hash: hash(&game), }).unwrap();
        }
    }
    
    fn get_it_done(&self, request:JobM) -> DoneM {
        let (mut game, player) = Worker::parse_request(request);
        let mut wins = Vec::new();
        let mut draws = Vec::new();
        let mut losses = Vec::new();
        let mut undecided = Vec::new();

        for mv in game.possible_moves(&player).iter() {
            match game.make_move(&player, mv.clone()) {
                Ok(Score::Won(_)) => wins.push(mv.clone()),
                Ok(Score::Remis(_)) => draws.push(mv.clone()),
                Ok(Score::Lost(_)) => losses.push(mv.clone()),
                Ok(Score::Undecided(_)) => undecided.push(mv.clone()),
                _ => (),
            }
            game.withdraw_move(&player, mv.clone());
        }

        if wins.len()>0 {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Won, hash: hash(&game), }).unwrap();
        } else if undecided.len()>0 {
            self.treat_undecided(undecided, game, &player);
        } else if draws.len()>0 {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Draw, hash: hash(&game), }).unwrap();
        } else if losses.len()>0 {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Lost, hash: hash(&game), }).unwrap();
        } else {
            self.query.send(QueryM { worker_id: self.worker_id, score: ScoreEntry::Draw, hash: hash(&game), }).unwrap();
        }
        DoneM { worker_id: self.worker_id }
    }

    fn new(worker_id: i32, query: Sender<QueryM>, record: Receiver<StoreM>, 
           ctl_in: Receiver<JobM>, ctl_out: Sender<DoneM>) -> Self {
        Worker {
            worker_id: worker_id,
            query: query,
            record: record,
            ctl_in: ctl_in,
            ctl_out: ctl_out,
        }
    }

    fn run(self) {
        let handle = thread::spawn(move || {
            while true {
                let request:JobM = self.ctl_in.recv().unwrap();
                let answer = self.get_it_done(request);
                self.ctl_out.send(answer).unwrap()
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

struct JobM {
    game: ConnectFour,
    player: Player,
}

struct DoneM {
    worker_id: i32,
}

impl BruteForceStrategy {
    pub fn new(nworker:i32) -> Self {
        let (query_s, query_r) = channel();
        let (ctl_out_s, ctl_out_r) = channel();

        let mut bfs = BruteForceStrategy { 
            workers: Vec::new(),
            query_receiver: query_r,
            record_sender: Vec::new(),
            done_receiver: ctl_out_r,
            job_sender: Vec::new(),
            control: Controller::new(),
        };
        for worker_id in 0..nworker {
            let (record_s, record_r) = channel();
            bfs.record_sender.push(record_s);
            let (ctl_in_s, ctl_in_r) = channel();
            bfs.job_sender.push(ctl_in_s);
            
            bfs.workers.push(Worker::new(
                worker_id,
                query_s.clone(),
                record_r,
                ctl_in_r,
                ctl_out_s.clone()
            ));
        };
        bfs
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
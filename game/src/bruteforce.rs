//pub mod generic;
use generic::{Game, Move, Player, Score, Strategy, Withdraw};
use connectfour::{Column, ConnectFour, NaiveStrategy};
use std::rc::Rc;
use std::cell::RefCell;


pub struct BruteForceStrategy {
    workers:Vec<WorkerRadio>,
    control:Controller,
    query_receiver: Receiver<QueryM>,
    done_receiver: Receiver<DoneM>,
    report_receiver: Receiver<InterestingM>,
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
//    InCalculation,
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

struct InterestingM {
    worker_id: i32,
    hash: Hash,
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
    record: Option<Receiver<StoreM>>,
    report: Sender<InterestingM>,
    ctl_in: Option<Receiver<JobM>>,
    ctl_out: Sender<DoneM>,
}

struct WorkerRadio {
    worker_id: i32,
    record: Sender<StoreM>,
    ctl_in: Sender<JobM>,
}

fn hash(game:&ConnectFour) -> Hash {
    Hash{hash:String::from(""),}
}

impl Worker {
    fn new(worker_id:i32,
            query:Sender<QueryM>,
            report:Sender<InterestingM>,
            ctl_out:Sender<DoneM>) -> Self {
        Worker {
            worker_id:worker_id,
            query:query,
            record:None,
            report:report,
            ctl_in:None,
            ctl_out:ctl_out,
        }
    }

    fn parse_request(request:JobM) -> (ConnectFour,Player) {
        (ConnectFour::new(), Player::White)
    }

    fn get_entry(&self, hash:Hash) -> ScoreEntry {
        ScoreEntry::Unknown
    }

    fn treat_undecided(&self,
            undecided: Vec<std::rc::Rc<dyn Move<Column>>>,
            mut game: Rc<RefCell<ConnectFour>>,
            player:&Player) {
        let mut wins = Vec::new();
        let mut draws = Vec::new();
        let mut losses = Vec::new();
        let mut unknown = Vec::new();
        let mut missing = Vec::new();
/* The state InCalculation doesn't make sense in a strictly unsynchronized aproach
   one cannot know when the record is changed... */
//        let mut incalculation = Vec::new();

        for mv in undecided.iter() {
            game.borrow_mut().make_move(player, mv.clone()).unwrap();
            match self.get_entry(hash(&game.borrow())) {
                ScoreEntry::Won => wins.push(mv.clone()),
                ScoreEntry::Lost => losses.push(mv.clone()),
                ScoreEntry::Draw => draws.push(mv.clone()),
//                ScoreEntry::InCalculation => incalculation.push(mv.clone()),
                ScoreEntry::Unknown => unknown.push(mv.clone()),
                ScoreEntry::Missing => missing.push(mv.clone()),
            }
            game.borrow_mut().withdraw_move(&player, mv.clone());
        }

/*        while let Some(mv) = incalculation.pop() {
            game.borrow_mut().make_move(player, mv.clone()).unwrap();
            match self.get_entry(hash(&game.borrow())) {
                ScoreEntry::Won => wins.push(mv.clone()),
                ScoreEntry::Lost => losses.push(mv.clone()),
                ScoreEntry::Draw => draws.push(mv.clone()),
                ScoreEntry::InCalculation => incalculation.insert(0, mv.clone()),
                ScoreEntry::Unknown => unknown.push(mv.clone()),
                ScoreEntry::Missing => missing.push(mv.clone()),
            }
            game.borrow_mut().withdraw_move(&player, mv.clone());
        }
*/
        if wins.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Won,
                hash: hash(&game.borrow()),
            }).unwrap();
            return
        } 
        while let Some(qm) = missing.pop() {
            let naive = NaiveStrategy {};
            if let (_, Some(score)) = naive.find_best_move(game.clone(), player, 4, false) {
                match score {
                    Score::Won(_) => {
                        self.query.send(QueryM {
                            worker_id: self.worker_id,
                            score: ScoreEntry::Won,
                            hash: hash(&game.borrow()),
                        }).unwrap();
                        return
                    },
                    Score::Remis(_) => draws.push(qm.clone()),
                    Score::Lost(_) => losses.push(qm.clone()),
                    Score::Undecided(_) => unknown.push(qm.clone()),
                }
            }
        }

        unknown.iter().map(|mv|{
            match game.borrow_mut().make_move(&player, mv.clone()) {
                Ok(_) => {
                    self.report.send(
                        InterestingM { 
                            worker_id: self.worker_id, 
                            hash: hash(&game.borrow()) 
                    }).unwrap();
                    game.borrow_mut().withdraw_move(&player, mv.clone());
                },
                Err(_) => {
                    panic!("shouldn't happen.");
                },
            };
        }).for_each(drop);

        if unknown.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Unknown,
                hash: hash(&game.borrow()),
            }).unwrap();
        } else if draws.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Draw,
                hash: hash(&game.borrow()),
            }).unwrap();
        } else if losses.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Lost,
                hash: hash(&game.borrow()),
            }).unwrap();
        }
    }
    
    fn get_it_done(&self, request:JobM) -> DoneM {
        let (game, player) = Worker::parse_request(request);
        let game = Rc::new(RefCell::new(game));
        let mut wins = Vec::new();
        let mut draws = Vec::new();
        let mut losses = Vec::new();
        let mut undecided = Vec::new();

        for mv in game.borrow().possible_moves(&player).iter() {
            match game.borrow_mut().make_move(&player, mv.clone()) {
                Ok(Score::Won(_)) => wins.push(mv.clone()),
                Ok(Score::Remis(_)) => draws.push(mv.clone()),
                Ok(Score::Lost(_)) => losses.push(mv.clone()),
                Ok(Score::Undecided(_)) => undecided.push(mv.clone()),
                _ => (),
            }
            game.borrow_mut().withdraw_move(&player, mv.clone());
        }

        if wins.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Won,
                hash: hash(&game.borrow()),
            }).unwrap();
        } else if undecided.len()>0 {
            self.treat_undecided(undecided, game, &player);
        } else if draws.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Draw,
                hash: hash(&game.borrow()),
            }).unwrap();
        } else if losses.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Lost,
                hash: hash(&game.borrow()),
            }).unwrap();
        } else {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: ScoreEntry::Draw,
                hash: hash(&game.borrow()),
            }).unwrap();
        }
        DoneM { worker_id: self.worker_id }
    }

    fn run(mut self) -> WorkerRadio {
        let (record_s, record_r) = channel();
        self.record = Some(record_r);
        let (ctl_in_s, ctl_in_r) = channel();
        //self.ctl_in = Some(ctl_in_r);
        let wr = WorkerRadio {
            worker_id: self.worker_id,
            ctl_in:ctl_in_s.clone(),
            record:record_s.clone(),
        };
        let handle = thread::spawn(move || {
            while true {
                match ctl_in_r.recv() {
                    Ok(request) => {
                        let answer = self.get_it_done(request);
                        self.ctl_out.send(answer).unwrap()
                    },
                    Err(e) => {
                        panic!("{}", e)
                    }
                }
            };
        });
        wr
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
        let (report_s, report_r) = channel();

        let mut bfs = BruteForceStrategy { 
            workers: Vec::new(),
            query_receiver: query_r,
            done_receiver: ctl_out_r,
            control: Controller::new(),
            report_receiver: report_r,
        };

        for worker_id in 0..nworker {
            bfs.workers.push(Worker::new(
                worker_id,
                query_s.clone(),
                report_s.clone(),
                ctl_out_s.clone()
            ).run());
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
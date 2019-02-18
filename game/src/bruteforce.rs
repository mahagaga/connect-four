//pub mod generic;
use generic::{Game, Move, Player, Score, Strategy, Withdraw};
use connectfour::{Column, ConnectFour, NaiveStrategy};
use std::rc::Rc;
use std::cell::RefCell;


/* 
 hash <-> game conversion:
---------------------------
the idea is to reduce the number of games in the game store
by neutralization of stones. some stones cannot make a difference in the game (anymore)
hence they must not be distinguished by color
the number of possible games should be drastically decreasing through this neturalization.
*/

// for now these are only dummy implementations with no actual neutralization
// TODO: implmentation
fn from_hash(hash: Hash) -> (ConnectFour, Player) {
    (ConnectFour::replicate(hash.hash), hash.player)
}

fn to_hash(game:&ConnectFour, player:&Player) -> Hash {
    Hash {
        hash: game.display(),
        player: player.clone()
    }
}

/*enum Eval {
    Available(Score),
    InCalculation,
}*/

use std::collections::HashMap;
pub struct Store {
    pub scores: HashMap<String,ScoreEntry>,
}

#[derive(Clone,Debug,PartialEq)]
struct Hash {
    hash: String,
    player: Player,
}

#[derive(Clone,Debug)]
pub enum ScoreEntry {
    Won,
    Lost,
    Draw,
//    InCalculation,
    Unknown,
    Missing,
}

impl Store {
    fn new() -> Self {
        Store {
            scores: HashMap::new(),
        }
    }
}

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

struct Worker {
    worker_id: usize,
    query: Sender<QueryM>,
    record: Option<Receiver<StoreM>>,
    report: Sender<InterestingM>,
}

impl Worker {
    fn new(worker_id:usize,
            query:Sender<QueryM>,
            report:Sender<InterestingM>) -> Self {
        Worker {
            worker_id:worker_id,
            query:query,
            record:None,
            report:report,
        }
    }

    fn parse_request(request:JobM) -> (ConnectFour,Player) {
        from_hash(request.hash)
    }

    fn get_entry(&self, hash:Hash) -> ScoreEntry {
        let q = QueryM {
            worker_id: self.worker_id,
            hash: hash,
            score: None,
            stop: false,
        };
        loop {
            if let Ok(_) = self.query.send(q.clone()) {
                match self.record {
                    Some(ref receiver) => {
                        if let Ok(score) = receiver.recv() {
                            if q.hash != score.hash {
                                panic!("that is not the answer to this question!")
                            }
                            return score.score;
                        } else {
                            println!("score receiver doesn't work (worker {})", self.worker_id)
                        }
                    },
                    None => panic!("worker {} without receiver", self.worker_id),
                };
            } else {
                println!("query sender doesn't work (worker {})", self.worker_id)
            }
            thread::sleep(std::time::Duration::new(5,0));
        }
    }

    fn treat_undecided(&self,
            undecided: Vec<std::rc::Rc<dyn Move<Column>>>,
            game: Rc<RefCell<ConnectFour>>,
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
            match self.get_entry(to_hash(&game.borrow(), player)) {
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
                score: Some(ScoreEntry::Won),
                hash: to_hash(&game.borrow(), player),
                stop: false,
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
                            score: Some(ScoreEntry::Won),
                            hash: to_hash(&game.borrow(), player),
                            stop: false,
                        }).unwrap();
                        return
                    },
                    Score::Remis(_) => draws.push(qm.clone()),
                    Score::Lost(_) => losses.push(qm.clone()),
                    Score::Undecided(_) => unknown.push(qm.clone()),
                }
            }
        }

        let job_hash = to_hash(&game.borrow(), player);
        unknown.iter().map(|mv|{
            let score = game.borrow_mut().make_move(&player, mv.clone());
            match score {
                Ok(_) => {
                    if let Err(e) = self.report.send(
                        InterestingM {
                            worker_id: self.worker_id,
                            from: Some(job_hash.clone()),
                            hash: Some(to_hash(&game.borrow(), player)),
                    }) {
                        println!("something happened during report of worker {}: {}", self.worker_id, e);
                        thread::sleep(std::time::Duration::new(5,0));
                    }
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
                score: Some(ScoreEntry::Unknown),
                hash: to_hash(&game.borrow(), player),
                stop: false,
            }).unwrap();
        } else if draws.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: Some(ScoreEntry::Draw),
                hash: to_hash(&game.borrow(), player),
                stop: false,
            }).unwrap();
        } else if losses.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: Some(ScoreEntry::Lost),
                hash: to_hash(&game.borrow(), player),
                stop: false,
            }).unwrap();
        }
    }
    
    fn get_it_done(&self, request:JobM) {
        let hash_clone = request.hash.clone();
        let (game, player) = Worker::parse_request(request);
        let game = Rc::new(RefCell::new(game));
        let mut wins = Vec::new();
        let mut draws = Vec::new();
        let mut losses = Vec::new();
        let mut undecided = Vec::new();

        let possibilities = game.borrow().possible_moves(&player);
        for mv in possibilities.iter() {
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
                score: Some(ScoreEntry::Won),
                hash: to_hash(&game.borrow(), &player),
                stop: false,
            }).unwrap();
        } else if undecided.len()>0 {
            self.treat_undecided(undecided, game, &player);
        } else if draws.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: Some(ScoreEntry::Draw),
                hash: to_hash(&game.borrow(), &player),
                stop: false,
            }).unwrap();
        } else if losses.len()>0 {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: Some(ScoreEntry::Lost),
                hash: to_hash(&game.borrow(), &player),
                stop: false,
            }).unwrap();
        } else {
            self.query.send(QueryM {
                worker_id: self.worker_id,
                score: Some(ScoreEntry::Draw),
                hash: to_hash(&game.borrow(), &player),
                stop: false,
            }).unwrap();
        }

        self.report.send(InterestingM {
            worker_id: self.worker_id,
            from: Some(hash_clone),
            hash: None, // i.e.: done
        }).unwrap();
    }

    fn run(mut self) -> (Sender<JobM>, Sender<StoreM>) {
        let (record_s, record_r) = channel();
        self.record = Some(record_r);
        let (ctl_in_s, ctl_in_r) = channel();
        //self.ctl_in = Some(ctl_in_r);
        let wr = (
            ctl_in_s.clone(),
            record_s.clone(),
        );
        
        thread::spawn(move || {
            loop {
                match ctl_in_r.recv() {
                    Ok(request) => {
                        self.get_it_done(request);
                    },
                    Err(e) => {
                        println!("worker {} receiver failed: {}", self.worker_id, e);
                        thread::sleep(std::time::Duration::new(5,0));
                    },
                }
            };
        });
        wr
    }
}

// worker -> store
#[derive(Clone, Debug)]
pub struct QueryM {
    worker_id: usize,
    hash: Hash,
    score: Option<ScoreEntry>, //Some() for put, None for get
    stop: bool,
}

#[allow(non_snake_case)]
pub fn STOP() ->QueryM {
    QueryM {
        worker_id: 0,
        hash: Hash{hash:String::from(""), player:Player::White},
        score: None, stop: true
    }
}

// store -> worker
struct StoreM {
    hash: Hash,
    score: ScoreEntry,
}

// worker -> control
struct InterestingM {
    worker_id: usize,
    from: Option<Hash>, //None for initial query
    hash: Option<Hash>, //Some() for dependency, None for done
}

// control -> worker
struct JobM {
    hash: Hash,
}

pub struct BruteForceStrategy {
    workers: Vec<Sender<JobM>>,
    report_receiver: Receiver<InterestingM>,
    pub store_handle: Option<std::thread::JoinHandle<Store>>,
    pub stopper: Sender<QueryM>,
}

impl BruteForceStrategy {
    pub fn new(nworker:usize) -> Self {
        let (query_s, query_r) = channel();
        let (report_s, report_r) = channel();

        let mut bfs = BruteForceStrategy { 
            workers: Vec::new(),
            report_receiver: report_r,
            store_handle: None,
            stopper: query_s.clone(),
        };

        let mut records = Vec::new();
        for worker_id in 0..nworker {
            let (job_sender, store_sender) = Worker::new(
                worker_id,
                query_s.clone(),
                report_s.clone()
            ).run();
            bfs.workers.push(job_sender);
            records.push(store_sender);
        };

        bfs.store_handle = Some(thread::spawn(move || {
            let mut store = Store::new();
            loop {
                match query_r.recv(){
                    Ok(query) => {
                        if query.stop { break; }
                        println!("query {:?}", query);
                        match query.score {
                            // lookup scores table and send value back - or 'Missing'
                            None => {
                                records[query.worker_id].send(StoreM{
                                    hash: query.hash.clone(), 
                                    score: match store.scores.get(&query.hash.hash) {
                                        None => ScoreEntry::Missing,
                                        Some(score_entry) => (*score_entry).clone(),
                                    }, 
                                }).unwrap();
                            },
                            // update scores table
                            Some(newscore) => {
                                store.scores.insert(
                                    query.hash.hash,
                                    newscore
                                );
                            },
                        };
                    },
                    Err(e) => {
                        println!("store receiver failed: {}", e);
                        thread::sleep(std::time::Duration::new(5,0));
                    },
                }
            }
            store
        }));

        bfs
    }
    
    pub fn pave_ground(&self,
            g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
            p: &Player,
            toplimit: u32) {
        
        self.workers[0].send(JobM {
            hash: Hash {
                hash: g.borrow().display(),
                player: p.clone(),
            },
        }).unwrap();
        
        let mut i:u32 = 0;
        loop {
            match self.report_receiver.recv() {
                Ok(interest) => {
                    println!("interest {:?} {:?}", interest.from, interest.hash);
                    match interest.hash {
                        Some(h) => {
                            self.workers[(i%self.workers.len()as u32) as usize].send(JobM { 
                            hash: h }).unwrap();
                        },
                        None => {
                            println!("{} done", interest.worker_id);
                        }
                    }
                },
                Err(e) => {
                    println!("report receiver failed {}", e);
                    thread::sleep(std::time::Duration::new(5,0));
                }
            }
            i+=1;
            if i>=toplimit {
                println!("toplimit {} reached", toplimit);
                break;
            }
        }
    }
}

impl Strategy<Column,Vec<Vec<Option<Player>>>> for BruteForceStrategy {
    fn evaluate_move(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                     p: &Player, mv: Rc<Move<Column>>) -> Result<f32,Withdraw> {
        NaiveStrategy{}.evaluate_move(g, p, mv)
    }
}
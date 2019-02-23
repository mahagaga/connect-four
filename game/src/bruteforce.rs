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
#[derive(Debug)]
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
use std::thread::JoinHandle;
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

    fn parse_request(request:Hash) -> (ConnectFour,Player) {
        from_hash(request)
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
            player:&Player,
            lookahead:u32) {
        let mut unknown = Vec::new();
        let mut missing = Vec::new();

        for mv in undecided.iter() {
            game.borrow_mut().make_move(player, mv.clone()).unwrap();
            match self.get_entry(to_hash(&game.borrow(), player)) {
                ScoreEntry::Won => return,
                ScoreEntry::Lost => (),
                ScoreEntry::Draw => (),
                ScoreEntry::Unknown => unknown.push(mv.clone()),
                ScoreEntry::Missing => missing.push(mv.clone()),
            }
            game.borrow_mut().withdraw_move(&player, mv.clone());
        }

        while let Some(qm) = missing.pop() {
            game.borrow_mut().make_move(&player, qm.clone()).unwrap();
            let naive = NaiveStrategy {};
            if let (Some(_), Some(score)) = naive.find_best_move(game.clone(), player.opponent(), lookahead as i32, false) {
                match score {
                    Score::Lost(_) => {
                        let winm = QueryM {
                            worker_id: self.worker_id,
                            score: Some(ScoreEntry::Won),
                            hash: to_hash(&game.borrow(), player),
                            stop: false,
                        };
                        game.borrow_mut().withdraw_move(&player, qm.clone());
                        self.query.send(winm).unwrap();
                        return
                    },
                    Score::Remis(_) => {
                        self.query.send(QueryM {
                            worker_id: self.worker_id,
                            score: Some(ScoreEntry::Draw),
                            hash: to_hash(&game.borrow(), player),
                            stop: false,
                        }).unwrap();
                    },
                    Score::Won(_) => {
                        self.query.send(QueryM {
                            worker_id: self.worker_id,
                            score: Some(ScoreEntry::Lost),
                            hash: to_hash(&game.borrow(), player),
                            stop: false,
                        }).unwrap();
                    },
                    Score::Undecided(_) => {
                        self.query.send(QueryM {
                            worker_id: self.worker_id,
                            score: Some(ScoreEntry::Unknown),
                            hash: to_hash(&game.borrow(), player),
                            stop: false,
                        }).unwrap();
                        unknown.push(qm.clone())
                    },
                }
            }
            game.borrow_mut().withdraw_move(&player, qm.clone());
        }

        let job_hash = to_hash(&game.borrow(), player);
        unknown.iter().map(|mv|{
            let score = game.borrow_mut().make_move(&player, mv.clone());
            match score {
                Ok(_) => {
                    let mut interests:Vec<InterestingM> = Vec::new();
                    let mut lost = false;
                    let opoptions = game.borrow().possible_moves(player.opponent());
                    for opmv in opoptions {
                        let opscore = game.borrow_mut().make_move(player.opponent(), opmv.clone());
                        match opscore {
                            Ok(Score::Won(_)) => {
                                game.borrow_mut().withdraw_move(player.opponent(), opmv.clone());
                                self.query.send(QueryM {
                                    worker_id: self.worker_id,
                                    score: Some(ScoreEntry::Lost),
                                    hash: to_hash(&game.borrow(), player),
                                    stop: false,
                                }).unwrap();
                                lost = true;
                                break;
                            },
                            Ok(Score::Undecided(_)) => {
                               interests.push(
                                    InterestingM {
                                        worker_id: self.worker_id,
                                        from: Some(job_hash.clone()),
                                        hash: Some(to_hash(&game.borrow(), player)),
                                });
                                game.borrow_mut().withdraw_move(player.opponent(), opmv);
                            },
                            Ok(_) => {
                                game.borrow_mut().withdraw_move(player.opponent(), opmv);
                            }
                            Err(_) => panic!("shouldn't happen."),
                        }
                    }
                    if !lost {
                        for interest in interests {
                            if let Err(e) = self.report.send(interest) {
                                println!("something happened during report of worker {}: {}", self.worker_id, e);
                                //thread::sleep(std::time::Duration::new(5,0));
                                panic!();
                            }
                        }
                    }
                    game.borrow_mut().withdraw_move(&player, mv.clone());
                },
                Err(_) => {
                    panic!("shouldn't happen.");
                },
            };
        }).for_each(drop);
    }
    
    fn get_it_done(&self, hash:Hash, lookahead:u32) {
        let hash_clone = hash.clone();
        let (game, player) = Worker::parse_request(hash);
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
            return
        } else if undecided.len()>0 {
            self.treat_undecided(undecided, game, &player, lookahead);
        }

        self.report.send(InterestingM {
            worker_id: self.worker_id,
            from: Some(hash_clone),
            hash: None, // i.e.: done
        }).unwrap();
    }

    fn run(mut self, lookahead:u32) -> (Sender<JobM>, JoinHandle<()>, Sender<StoreM>) {
        let (record_s, record_r) = channel();
        self.record = Some(record_r);
        let (ctl_in_s, ctl_in_r):(Sender<JobM>,Receiver<JobM>) = channel();
        //self.ctl_in = Some(ctl_in_r);
        let job_sender = ctl_in_s.clone();
        let store_sender = record_s.clone();
        
        let handle = thread::spawn(move || {
            loop {
                match ctl_in_r.recv() {
                    Ok(request) => {
                        if request.day_call { break; } else {
                            self.get_it_done(request.hash, lookahead);
                        }
                    },
                    Err(e) => {
                        println!("worker {} receiver failed: {}", self.worker_id, e);
                        thread::sleep(std::time::Duration::new(5,0));
                    },
                }
            };
        });
        (job_sender, handle, store_sender)
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
    day_call:bool, // for sending the worker home
}

pub struct BruteForceStrategy {
    workers: Vec<Sender<JobM>>,
    work_handles: Vec<JoinHandle<()>>,
    report_receiver: Receiver<InterestingM>,
    store_handle: Option<std::thread::JoinHandle<Store>>,
    stopper: Sender<QueryM>,
}

impl BruteForceStrategy {
    pub fn new(nworker:usize,
            lookahead: u32) -> Self {
        let (query_s, query_r) = channel();
        let (report_s, report_r) = channel();

        let mut bfs = BruteForceStrategy { 
            workers: Vec::new(),
            work_handles: Vec::new(),
            report_receiver: report_r,
            store_handle: None,
            stopper: query_s.clone(),
        };

        let mut records = Vec::new();
        for worker_id in 0..nworker {
            let (job_sender, join_handle, store_sender) = Worker::new(
                worker_id,
                query_s.clone(),
                report_s.clone()
            ).run(lookahead);
            bfs.workers.push(job_sender);
            bfs.work_handles.push(join_handle);
            records.push(store_sender);
        };

        bfs.store_handle = Some(thread::spawn(move || {
            let mut store = Store::new();
            loop {
                match query_r.recv(){
                    Ok(query) => {
                        if query.stop { break; }
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
                                println!("query {:?} {:?}", query.hash.hash, newscore);
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
            day_call: false,
        }).unwrap();
        
        let mut jobs = 0;
        let mut done = 0;
        loop {
            match self.report_receiver.recv() {
                Ok(interest) => {
                    println!("interest {:?} {:?}", interest.from, interest.hash);
                    match interest.hash {
                        Some(h) => {
                            self.workers[(jobs%self.workers.len()as u32) as usize].send(JobM { 
                            hash: h, day_call: false }).unwrap();
                            jobs += 1;
                        },
                        None => {
                            println!("{} done", interest.worker_id);
                            done += 1;
                        }
                    }
                },
                Err(e) => {
                    println!("report receiver failed {}", e);
                    thread::sleep(std::time::Duration::new(5,0));
                }
            }
            if done >= jobs {
                println!("all jobs done {}", done);
                break;
            }
            if done>=toplimit {
                println!("toplimit {} reached", toplimit);
                break;
            }
        }
    }

    pub fn collect_store(mut self) -> Store {
        self.workers.iter().for_each(|w| {
            w.send(JobM{
                hash:Hash{hash:String::from(""),player:Player::White},
                day_call:true
            }).unwrap();
        });
        loop {
            match self.work_handles.pop() {
                None => { break; },
                Some(wh) => wh.join().unwrap(),
            }
        }
        self.stopper.send(STOP()).unwrap();
        self.store_handle.unwrap().join().unwrap()
    }
}

impl Strategy<Column,Vec<Vec<Option<Player>>>> for BruteForceStrategy {
    fn evaluate_move(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                     p: &Player, mv: Rc<Move<Column>>) -> Result<f32,Withdraw> {
        NaiveStrategy{}.evaluate_move(g, p, mv)
    }
}
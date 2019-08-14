//pub mod generic;
use generic::{Game,Move,Player,Score,Strategy,Withdraw};
use connectfour::{Column,ConnectFour,ConnectFourMove};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use std::cmp;
use std::sync::mpsc::{channel,Sender,Receiver};
use std::thread;


//#################################################################################################
// specifically Connect Four
//#################################################################################################


//### connect four strategy #######################################################################

type GameHash = i128;

fn hash_from_game(game:Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>) -> GameHash {
    let mut s = 0;
    let mut f = 1;
    let mut ci = 0;
    let base:i128 = 4;
    for c in game.borrow().state() {
        for x in c {
            match x {
                Some(p) => match p {
                    Player::White => { s += 1 * f; },
                    Player::Black => { s += 2 * f; },
                },
                None => (),
            }
            f *= base;
        }
        ci += 1;
        f = base.pow(ConnectFour::height() as u32 * ci);
    }
    s
}

fn game_from_hash(hash:GameHash) -> ConnectFour {
    let mut game = ConnectFour::new();
    let mut h = hash;
    let base:i128 = 4;
    for ci in 0..ConnectFour::width() {
        let col = Column::from_usize(ci);
        let mut cr = h % base.pow(ConnectFour::height() as u32);
        for _ri in 0..ConnectFour::height() {
            let stone = cr % base;
            match stone {
                1 => { game.drop_stone(&Player::White, col.clone()).unwrap(); },
                2 => { game.drop_stone(&Player::Black, col.clone()).unwrap(); },
                0 => break,
                _ => (),
            }
            cr = cr / base;
        }
        h = h / base.pow(ConnectFour::height() as u32);
    }
    game
}

pub struct GameRecord {
    state: GameState,
}
pub enum GameState {
    Locked,
    Decided(Score, Option<Column>),
    Undecided,
}
pub struct BruteForceStrategy {
    pub nworkers: usize,
}

enum Cell {
    M, //my stone
    O, //opponent's stone
    N, //no stone, empty cell
    D, //dead cell, cannot be part of any four connected stones
}

impl BruteForceStrategy {
    #[allow(dead_code)]
    fn display_efield(&self, ef: &Vec<Vec<Cell>>) {
        for j in (0..ConnectFour::height()).rev() {
            for i in 0..ConnectFour::width() {
                print!("{}", match ef[i][j] {
                    Cell::N => ".",
                    Cell::M => "m",
                    Cell::O => "o",
                    Cell::D => ":",
                })
            }
            println!("|")
        }
    }

    #[allow(dead_code)]
    fn efield_counting(&self, ef: &Vec<Vec<Cell>>, ns: Vec<usize>, ms: Vec<usize>)
    -> ((i32, i32, i32,), (i32, i32, i32)) {
        let mut o_count = 0;
        let mut no_count = 0;
        let mut m_count = 0;
        let mut nm_count = 0;
        let mut count = 0;
        let mut first_opponent:Option<i32> = None;
        let mut first_mine:Option<i32> = None;
        for (i,j) in ns.iter().zip(ms.iter()) {
            match ef[*i][*j] {
                Cell::M => { if let None = first_mine { first_mine = Some(count); }
                             if let None = first_opponent { m_count += 1; }},
                Cell::O => { if let None = first_opponent { first_opponent = Some(count); }
                             if let None = first_mine { o_count += 1; }},
                Cell::N => { if let None = first_mine { no_count += 1; }
                             if let None = first_opponent { nm_count += 1; }},
                Cell::D => { break; },
            }
            count += 1;
        }
        if let None = first_mine { first_mine = Some(count); }
        if let None = first_opponent { first_opponent = Some(count); }
        match first_mine {
            Some(fm) => match first_opponent {
                Some(fo) => ((fo,m_count,nm_count),(fm,o_count,no_count)),
                _ => ((0,0,0),(0,0,0))
            }
            _ => ((0,0,0),(0,0,0))
        }
    }
}


impl Strategy<Column,Vec<Vec<Option<Player>>>> for BruteForceStrategy {
    
    fn evaluate_move(&self, g: Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
                     p: &Player, mv: Rc<dyn Move<Column>>) 
    -> Result<f32, Withdraw> {
        let n = mv.data().to_usize();
        let m = g.borrow().state()[n].len();
        if m >= ConnectFour::height() { return Err(Withdraw::NotAllowed); }

        // fill evaluation field with empty cells
        let mut efield = Vec::with_capacity(ConnectFour::width());
        for _ in 0..ConnectFour::width() {
            let mut ecol = Vec::with_capacity(ConnectFour::height());
            for _ in 0..ConnectFour::height() {
                ecol.push(Cell::N);
            }
            efield.push(ecol);
        }

        // copy current state into evaluation field
        let black = |player: &Player| { match player { Player::Black => Cell::M, Player::White => Cell::O }};
        let white = |player: &Player| { match player { Player::White => Cell::M, Player::Black => Cell::O }};
        let mut i:usize = 0;
        for c in g.borrow().state() { // that's the current Connect Four field
            let mut j:usize = 0;
            for f in c {
                match f {
                    Some(Player::Black) => efield[i][j] = black(p),
                    Some(Player::White) => efield[i][j] = white(p),
                    None => (),
                }
                j += 1;
            }
            i += 1;
        }
        
        Ok(0.5)
    }

    fn find_best_move(&self, 
            g: Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
            p: &Player,
            moves_ahead: i32,
            // in the brute force context, game evaluation is irrelevant, 
            // because it we have our own find_best_move implementation
            _game_evaluation: bool
        ) -> (Option<Rc<dyn Move<Column>>>, Option<Score>) {
        
        let (conductor, receiver) = Conductor::init_conductor_and_band();
        conductor.claim_public_interest(g);
        let (column, score) = self.await_verdict(receiver);
        
        (Some(Rc::new(ConnectFourMove{ data:column })), Some(score))
    }
}
impl BruteForceStrategy {
    pub fn default() -> Self {
        BruteForceStrategy {
            nworkers: 3,
        }
    }

    fn find_distinctive_moves(&self, 
            g: Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
            p: &Player,
            moves_ahead: i32,
        ) -> HashMap<Column,Score> {

        let mut scoremap = HashMap::new();
        
        let options = g.borrow().possible_moves(p);
        for mv in options.into_iter() {
            let score = g.borrow_mut().make_move(p, Rc::clone(&mv));
                                 
            match score {
                Ok(score) => match score {
                    Score::Won(in_n) => {
                        //println!("{}", &g.borrow().display());
                        //println!("{:?} wins with {:?} in {}", p, mv.display(), in_n);
                        g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
                        scoremap.insert((*Rc::clone(&mv)).data().clone(), score);
                        return scoremap;
                    },
                    Score::Undecided(pv) => {  },
                    _ => { scoremap.insert((*Rc::clone(&mv)).data().clone(), score); },
                },
                Err(_) => (),//return Err(_),
            }
            g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
        }
        scoremap
    }

    fn await_verdict(&self,
            receiver:Receiver<Verdict>
        ) -> (Column, Score) {
        (Column::One, Score::Undecided(0.0))
    }
}

fn from_move(mv:ConnectFourMove) -> Column {
    Column::One
}

struct Verdict {
    score: Score,
    column: Option<Column>,
}

pub struct Interest {
    interested: Option<GameHash>,
    interesting: Option<GameHash>,
    worker_id: Option<usize>,
}

pub struct Conductor {
    sender:Sender<Interest>,
}

impl Conductor {
    fn init_conductor_and_band () -> (Self, Receiver<Verdict>) {
        let (itx, interests) = channel::<Interest>();
        let interest_sender = itx.clone();
        let (final_verdict, rx) = channel::<Verdict>();
        
        let game_store = Arc::new(Mutex::new(HashMap::<GameHash,GameRecord>::new()));

thread::spawn(move|| {
    let mut interest_store:HashMap<GameHash,Vec<GameHash>> = HashMap::new();
    let mut workers:Vec<Worker> = vec![0; 4]
        .into_iter()
        .map(|i| Worker::spawn_worker(i, itx.clone(), game_store.clone()))
        .collect();

    loop {
        if let Ok(interest) = interests.recv() {
            match (interest.interesting, interest.interested) {
                // worker has finished job
                (None, Some(interested))=> {
                    // note changed job pendencies
                    let worker_id = interest.worker_id.unwrap();
                    workers.get_mut(worker_id).unwrap().pending_jobs -= 1;

                    // submit new jobs if this game is decided now
                    let finished = interested;
                    let gs = game_store.lock().unwrap();
                    match (*gs).get(&finished) {
                        Some(record) => {
                            match &record.state {
                                GameState::Decided(score, column) => {
                                    if let Some(interested) = interest_store.remove(&finished) {
                                        for jobhash in interested.into_iter() {
                                            let wid = (&workers).into_iter()
                                                .min_by_key(|w| w.pending_jobs).unwrap().id;
                                            let worker = workers.get_mut(wid).unwrap();
                                            worker.job_box.send(jobhash).unwrap();
                                            worker.pending_jobs += 1;
                                        }
                                    } else {
                                        for w in workers {
                                            w.job_box.send(-1).unwrap();
                                        }
                                        final_verdict.send(Verdict{
                                            score: score.clone(),
                                            column: column.clone(),
                                        }).unwrap();
                                        break;
                                    }
                                },
                                GameState::Locked => panic!("shouldn't happen!?"),
                                GameState::Undecided => (),
                            }
                        },
                        None => panic!("game should have a record!"),
                    }
                },
                // claimed interest
                (Some(interesting), parent) => {
                    if let Some(interested) = parent {
                        if let Some(record) = interest_store.get_mut(&interesting) {
                            record.push(interested);
                        } else {
                            interest_store.insert(interesting, vec![interested]);
                        }
                    }
                    let wid = (&workers).into_iter().min_by_key(|w| w.pending_jobs).unwrap().id;
                    let worker = workers.get_mut(wid).unwrap();
                    worker.job_box.send(interesting).unwrap();
                    worker.pending_jobs += 1;
                },
                (None, None) => panic!("doesn't make sense"),
            }         
        }
    }
});
        (Conductor{sender:interest_sender}, rx)
    }

    fn claim_public_interest(&self,
            g: Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>
        ) {
        let hash = hash_from_game(g);
        let sender = self.sender.clone();
        sender.send(Interest{
            interesting:Some(hash),
            interested:None,
            worker_id:None,
        }).unwrap();
    }
}

pub struct Worker {
    pending_jobs: u128,
    job_box: Sender<GameHash>,
    id: usize,
}

impl Worker {
    fn do_the_job(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            interest:&Sender<Interest>,
            hash:GameHash) {
        if !Worker::lock_hash(&game_store, hash) {
            return ();
        }
        let mut game = game_from_hash(hash);
        let decision = GameState::Undecided;
        Worker::unlock_hash(&game_store, hash, decision);
    }
    fn lock_hash(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            hash:GameHash) -> bool {
        let mut gs = game_store.lock().unwrap();
        if let Some(record) = (*gs).get(&hash) {
            match record.state {
                GameState::Undecided => {(*gs).insert(hash, GameRecord {
                    state: GameState::Locked,
                }); },
                _ => return false,
            }
        } else {
            (*gs).insert(hash, GameRecord { 
                state: GameState::Locked,
            });
        }
        true
    }
    fn unlock_hash(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            hash:GameHash,
            state:GameState) {
        let mut gs = game_store.lock().unwrap();
        (*gs).insert(hash, GameRecord { 
            state: state,
        });
    }

    fn spawn_worker(
            wid:usize,
            interest:Sender<Interest>,
            game_store:Arc<Mutex<HashMap<GameHash,GameRecord>>>) -> Worker {
        let (tx,jobs) = channel::<GameHash>();

thread::spawn(move|| {
    loop {
        match jobs.recv() {
            Err(e) => { println!("Job receive error - {}", e); }
            Ok(-1) => { break; },
            Ok(hash) => {
                Worker::do_the_job(&game_store, &interest, hash);
                interest.send(Interest{
                    interested: Some(hash), interesting: None, worker_id: Some(wid),
                });
            },
        }
    }
});
        Worker {
            pending_jobs:0,
            job_box:tx,
            id:wid
        }
    }
}

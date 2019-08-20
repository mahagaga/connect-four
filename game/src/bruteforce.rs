//pub mod generic;
use generic::{Game,Move,Player,Score,Strategy,Withdraw};
use connectfour::{Column,ConnectFour,ConnectFourMove};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
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
                    Player::Gray => { s += 2 * f; },
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
                3 => { game.drop_stone(&Player::Gray, col.clone()).unwrap(); },
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
        let black = |player: &Player| { match player {
            Player::Black => Cell::M, Player::White => Cell::O, Player::Gray => Cell::D, }};
        let white = |player: &Player| { match player {
            Player::White => Cell::M, Player::Black => Cell::O, Player::Gray => Cell::D, }};
        let mut i:usize = 0;
        for c in g.borrow().state() { // that's the current Connect Four field
            let mut j:usize = 0;
            for f in c {
                match f {
                    Some(Player::Black) => efield[i][j] = black(p),
                    Some(Player::White) => efield[i][j] = white(p),
                    Some(Player::Gray) => efield[i][j] = Cell::D,
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
        
        let (conductor, receiver) = Conductor::init_conductor_and_band(moves_ahead, p);
        conductor.claim_public_interest(g);
        let (column, score) = self.await_verdict(receiver);
        
        (Some(Rc::new(ConnectFourMove{ data:column })), Some(score))
    }
}

impl BruteForceStrategy {
    pub fn new(nworkers:usize) -> Self {
        BruteForceStrategy {
            nworkers: nworkers,
        }
    }

    fn await_verdict(&self,
            receiver:Receiver<Verdict>
        ) -> (Column, Score) {
        (Column::One, Score::Undecided(0.0))
    }
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
    fn init_conductor_and_band (moves_ahead:i32, p:&Player) -> (Self, Receiver<Verdict>) {
        let (itx, interests) = channel::<Interest>();
        let interest_sender = itx.clone();
        let (final_verdict, rx) = channel::<Verdict>();
        
        let game_store = Arc::new(Mutex::new(HashMap::<GameHash,GameRecord>::new()));
        let player = p.clone();

thread::spawn(move|| {
    let mut interest_store:HashMap<GameHash,Vec<GameHash>> = HashMap::new();
    let mut workers:Vec<Worker> = vec![0; 4]
        .into_iter()
        .map(|i| Worker::spawn_worker(i, itx.clone(), moves_ahead, game_store.clone()))
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
                                            worker.job_box.send((jobhash, player.clone())).unwrap();
                                            worker.pending_jobs += 1;
                                        }
                                    } else {
                                        for w in workers {
                                            w.job_box.send((-1, Player::Black)).unwrap();
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
                    worker.job_box.send((interesting, player.clone())).unwrap();
                    worker.pending_jobs += 1;
                },
                (None, None) => panic!("doesn't make sense"),
            }         
        }
    }
});
        (Conductor{sender:interest_sender,}, rx)
    }

    fn claim_public_interest(&self,
            g: Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
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
    job_box: Sender<(GameHash,Player)>,
    id: usize,
}

impl Worker {
    fn two_moves_ahead_inquiry(
        game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
        g:Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
        p:&Player,
    ) -> GameState {
        let options = g.borrow().possible_moves(p);
        if options.is_empty() { // no possible moves left: stalemate
            return GameState::Decided(Score::Remis(0), None);
        }

        let mut draw_moves = Vec::<(Score,Column)>::new();
        let mut doomed_moves = Vec::<(Score,Column)>::new();
        let mut open_moves = Vec::<GameHash>::new();

        for mv in options.into_iter() {
            let score = g.borrow_mut().make_move(p, Rc::clone(&mv));
            match score {
                Ok(score) => match score {
                    // found a winning move: immediate return
                    Score::Won(in_n) => {
                        g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
                        return GameState::Decided(Score::Won(in_n+1), Some(mv.data().clone()));
                    },
                    // found an undecided move, winning is still an option: let opponent make a move
                    Score::Undecided(_) => {
                        let anti_options = g.borrow().possible_moves(p.opponent());
                        if anti_options.is_empty() { // no possible moves left: stalemate
                            draw_moves.push((Score::Remis(1), mv.data().clone())); ;
                        } else {
                            let mut anti_draw_moves = Vec::<(Score,Column)>::new();
                            let mut anti_doomed_moves = Vec::<(Score,Column)>::new();
                            let mut anti_open_moves = Vec::<GameHash>::new();

                            for anti_mv in anti_options.into_iter() {
                                let anti_score = g.borrow_mut().make_move(p.opponent(), Rc::clone(&anti_mv));
                                match anti_score {
                                    Ok(score) => match score {
                                        Score::Won(in_n) => { // opponent has a winning move: losing
                                            doomed_moves.push((Score::Lost(in_n+2), mv.data().clone()));
                                            g.borrow_mut().withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                                            break;
                                        },
                                        Score::Lost(in_n) => { anti_doomed_moves.push((Score::Lost(in_n+1), mv.data().clone())); },
                                        Score::Remis(in_n) => { anti_draw_moves.push((Score::Remis(in_n+1), mv.data().clone())); },
                                        Score::Undecided(_) => { // unclear from the bord: check game store
                                            let hash = hash_from_game(g.clone());
                                            let gs = game_store.lock().unwrap();
                                            if let Some(record) = (*gs).get(&hash) {
                                                match &record.state {
                                                    GameState::Decided(record_score,_) => match record_score {
                                                        Score::Lost(in_n) => { // opponent can reach a lost game: losing
                                                            doomed_moves.push((Score::Lost(in_n+1), mv.data().clone()));
                                                            g.borrow_mut().withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                                                            break;
                                                        },
                                                        Score::Remis(in_n) => { anti_draw_moves.push((Score::Remis(in_n+1), mv.data().clone())); },
                                                        Score::Won(in_n) => { anti_doomed_moves.push((Score::Lost(in_n+1), mv.data().clone())); },
                                                        Score::Undecided(_) => { anti_open_moves.push(hash); },
                                                    }
                                                    _ => { anti_open_moves.push(hash); },
                                                }
                                            } else { anti_open_moves.push(hash); }
                                        },
                                    },
                                    Err(_) => panic!("unexpected error in anti move"),
                                }
                                g.borrow_mut().withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                            }

                            // if opponent has a winning move, the loop was broken and the triage below is not executed
                            if !anti_open_moves.is_empty() { // best opponent move is undecided
                                for interesting_hash in anti_open_moves.into_iter() {
                                    open_moves.push(interesting_hash);
                                }
                            } else if !anti_draw_moves.is_empty() { // best opponent move leads to a draw
                                let (score, _) = anti_draw_moves.first().unwrap();
                                if let Score::Remis(in_n) = score {
                                    draw_moves.push((Score::Remis(in_n+1), mv.data().clone()));
                                }
                            } else if !anti_doomed_moves.is_empty() { // opponent can only lose
                                let (score, _) = anti_doomed_moves.first().unwrap();
                                if let Score::Lost(in_n) = score {
                                    g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
                                    return GameState::Decided(Score::Won(in_n+1), Some(mv.data().clone()));
                                }
                            }

                        }
                    },
                    Score::Remis(in_n) => { draw_moves.push((Score::Remis(in_n+1), mv.data().clone())); },
                    Score::Lost(in_n) => { doomed_moves.push((Score::Lost(in_n+1), mv.data().clone())); },
                },
                Err(_) => panic!("unexpected error in move"),
            }
            g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
        }

        // if there is a winning move, it was returned already
        if !open_moves.is_empty() { // best move is yet undecided
            return GameState::Undecided;
        } else if !draw_moves.is_empty() { // best move is a draw
            let (score, col) = draw_moves.first().unwrap();
            return GameState::Decided(score.clone(), Some(col.clone()));
        } else if !doomed_moves.is_empty() { // all is lost
            let (score, col) = draw_moves.first().unwrap();
            return GameState::Decided(score.clone(), Some(col.clone()));
        }
        GameState::Undecided
    }

    fn game_simulation(
        moves_ahead:i32,
        g:Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
        p:&Player
    ) -> GameState {
        return GameState::Undecided
    }
    fn claim_interests(
        interest:&Sender<Interest>,
        g:Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
        p:&Player
    ) {
        ()
    }
    fn do_the_job(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            moves_ahead:i32,
            interest:&Sender<Interest>,
            hash:GameHash,
            p:&Player) {
        // return if game is locked or decided
        if !Worker::lock_hash(&game_store, hash) {
            return ();
        }

        let game = Rc::new(RefCell::new(game_from_hash(hash)));

        // 1. try to find a solution from the game store two moves ahead
        match Worker::two_moves_ahead_inquiry(&game_store, game.clone(), p) {
            GameState::Decided(verdict, mv) => { 
                return Worker::unlock_hash(&game_store, hash, GameState::Decided(verdict, mv));
            },
            GameState::Locked => panic!("unexpected state at this state"),
            GameState::Undecided => (),
        }
        // 2. try to find a solution from game simulation
        match Worker::game_simulation(moves_ahead, game.clone(), p) {
            GameState::Decided(verdict, mv) => { 
                return Worker::unlock_hash(&game_store, hash, GameState::Decided(verdict, mv));
            },
            GameState::Locked => panic!("unexpected state at this state"),
            GameState::Undecided => {
        // 3. claim interest for the remaining undecided games two moves ahead
                Worker::claim_interests(interest, game.clone(), p);
                return Worker::unlock_hash(&game_store, hash, GameState::Undecided);
            },
        }
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
            moves_ahead:i32,
            game_store:Arc<Mutex<HashMap<GameHash,GameRecord>>>) -> Worker {
        let (tx,jobs) = channel::<(GameHash,Player)>();
        let moves_ahead = moves_ahead;

thread::spawn(move|| {
    loop {
        match jobs.recv() {
            Err(e) => { println!("Job receive error - {}", e); }
            Ok((-1,_)) => { break; },
            Ok((hash,p)) => {
                Worker::do_the_job(&game_store, moves_ahead, &interest, hash, &p);
                interest.send(Interest{
                    interested: Some(hash), interesting: None, worker_id: Some(wid),
                }).unwrap();
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

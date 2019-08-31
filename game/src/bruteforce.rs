//pub mod generic;
use generic::{Game,Move,Player,Score,Strategy,Withdraw};
use connectfour::{Column,ConnectFour,ConnectFourMove,ConnectFourStrategy};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use std::sync::mpsc::{channel,Sender,Receiver};
use std::thread;
use std::time::Duration;


//#################################################################################################
// specifically Connect Four
//#################################################################################################


//### connect four strategy #######################################################################

type GameHash = i128;

fn hash_from_game(game:Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>) -> GameHash {
    hash_from_state(game.borrow().state())
}

fn hash_from_state(state:&Vec<Vec<Option<Player>>>) -> GameHash {
    let mut s = 0;
    let mut f = 1;
    let mut ci = 0;
    let base:i128 = 4;
    for c in state {
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
#[derive(Debug)]
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
        let principal = hash_from_game(g.clone());
        let (conductor, receiver) = Conductor::init_conductor_and_band(principal, moves_ahead, p, self.nworkers);
        conductor.claim_public_interest(g);
        let (column, score) = self.await_verdict(receiver);
        match column {
            None =>  (None, Some(score)),
            Some(column) => (Some(Rc::new(ConnectFourMove{ data:column })), Some(score)),
        }
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
        ) -> (Option<Column>, Score) {
        loop {
            if let Ok(verdict) = receiver.recv() {
                return (verdict.column, verdict.score);
            } else {
               // println!("await verdict");
                thread::sleep(Duration::from_millis(10));
            }
        }
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
    fn dump_store(
        game_store:Arc<Mutex<HashMap<GameHash,GameRecord>>>,
        principal:GameHash,
        p:&Player,
    ) {
//print!("+_");
        let gs = game_store.lock().unwrap();
//print!("_+");

        // TODO: full implementation
        if let Some(record) = (*gs).get(&principal) {
             println!("game {} state {:?}", principal, record.state);
        }

        // print the next couple of moves

        let g = Rc::new(RefCell::new(game_from_hash(principal)));
        println!("{}", g.borrow().display());
        
        let options = g.borrow().possible_moves(p);
        if options.is_empty() { // no possible moves left: stalemate
            return ();
        }

        for mv in options.into_iter() {
            let score_result = g.borrow_mut().make_move(p, Rc::clone(&mv));
            match score_result {
                Ok(score) => match score {
                    // found a winning move: immediate return
                    Score::Won(in_n) => println!("immediate winner in {}: {:?}", in_n, mv.data()),
                    Score::Remis(in_n) => println!("immediate draw in {}: {:?}", in_n, mv.data()),
                    Score::Lost(in_n) => println!("immediate loser in {}: {:?}", in_n, mv.data()),

                    // found an undecided move, winning is still an option: let opponent make a move
                    Score::Undecided(_) => {
                        println!("option {:?}", mv.data());
                        let anti_options = g.borrow().possible_moves(p.opponent());

                        for anti_mv in anti_options.into_iter() {
                            let anti_score = g.borrow_mut().make_move(p.opponent(), Rc::clone(&anti_mv));
                            match anti_score {
                                Ok(score) => match score {
                                    Score::Won(in_n) => println!("- consequent winner in {}: {:?}", in_n, anti_mv.data()),
                                    Score::Lost(in_n) => println!("- consequent loser in {}: {:?}", in_n, anti_mv.data()),
                                    Score::Remis(in_n) => println!("- consequent draw in {}: {:?}", in_n, anti_mv.data()),
                                    Score::Undecided(_) => { // unclear from the bord: check game store
                                        let hash = hash_from_game(g.clone());
                                        if let Some(record) = (*gs).get(&hash) {
                                            match &record.state {
                                                GameState::Decided(record_score,_) => match record_score {
                                                    Score::Lost(in_n) => println!("- stored loser in {}: {:?} -> {}", in_n, anti_mv.data(), hash),
                                                    Score::Remis(in_n) => println!("- stored draw in {}: {:?} -> {}", in_n, anti_mv.data(), hash),
                                                    Score::Won(in_n) => println!("- stored winner in {}: {:?} -> {}", in_n, anti_mv.data(), hash),
                                                    Score::Undecided(_) => println!("- STORED DECIDED UNDECIDED (?) {:?} -> {}", anti_mv.data(), hash),
                                                }
                                                state => println!("- stored undecided ({:?}) {:?} -> {}", state, anti_mv.data(), hash),
                                            }
                                        } else { println!("- unrecorded {:?} -> {}", anti_mv.data(), hash); }
                                    },
                                },
                                Err(_) => panic!("unexpected error in anti move"),
                            }
                            g.borrow_mut().withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                        }
                                        
                    },
                },
                Err(_) => panic!("unexpected error in move"),
            }
            g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
        }
    }

    fn init_conductor_and_band (principal:GameHash, moves_ahead:i32, p:&Player, nworkers:usize) -> (Self, Receiver<Verdict>) {
        let (itx, interests) = channel::<Interest>();
        let interest_sender = itx.clone();
        let (final_verdict, rx) = channel::<Verdict>();
        
        let game_store = Arc::new(Mutex::new(HashMap::<GameHash,GameRecord>::new()));
        let player = p.clone();
        let principal:GameHash = principal;

thread::spawn(move|| {
    let mut interest_store:HashMap<GameHash,Vec<GameHash>> = HashMap::new();
    let mut workers:Vec<Worker> = Vec::new();
    for i in 0..nworkers {
        workers.push(Worker::spawn_worker(i, itx.clone(), moves_ahead, game_store.clone()));
    }

    loop {
        if let Ok(interest) = interests.recv() {
            match (interest.interesting, interest.interested) {
                // worker has finished job
                (None, Some(interested)) => {
                    // note changed job pendencies
                    let worker_id = interest.worker_id.unwrap();
                    workers.get_mut(worker_id).unwrap().pending_jobs -= 1;

                    // submit new jobs if this game is decided now
                    let finished = interested;
                    let record;
                    {
//print!("+:");
                        let gst = game_store.lock().unwrap();
                        record = match (*gst).get(&finished) {
                            Some(x) => {
                                match &x.state {
                                    GameState::Decided(score, column) => Some((score.clone(), column.clone())),
                                    GameState::Locked => {
// debug
//println!("{} must have been picked up again already", &finished);
//
                                        None
                                    }
                                    GameState::Undecided => None,
                                }
                            },
                            None => panic!("game should have a record!"),
                        };
//print!(":+"); 
                    }
                    if let Some((score, column)) = record {
                        if let Some(parents) = interest_store.remove(&finished) {
                            for jobhash in parents.into_iter() {
                                let wid = (&workers).into_iter()
                                    .min_by_key(|w| w.pending_jobs).unwrap().id;
                                let worker = workers.get_mut(wid).unwrap();
                                loop {
                                    match worker.job_box.send((jobhash, player.clone())) {
                                        Ok(_) => break,
                                        Err(_) => {
                                            println!("worker? {}", wid);
                                            thread::sleep(Duration::from_millis(1));
                                        },
                                    }
                                }
                                worker.pending_jobs += 1;
                            }
                        } else if finished == principal {
                            for w in workers {
                                loop {
                                    match w.job_box.send((-1, Player::Black)) {
                                        Ok(_) => break,
                                        Err(_) => {
                                            println!("anonymous worker?");
                                            thread::sleep(Duration::from_millis(1));
                                        },
                                    }
                                }
                            }
                            Conductor::dump_store(game_store.clone(), principal, &player);
                            final_verdict.send(Verdict{
                                score: score.clone(),
                                column: column.clone(),
                            }).unwrap();
                            break;
                        }
                    }
                },
                // claimed interest
                (Some(interesting), parent) => {
                    if let Some(interested) = parent {
                        if let Some(record) = interest_store.get_mut(&interesting) {
                            // here, a HashSet would certainly be useful instead of a Vector
                            // but since memory is the likely bottleneck AND it's assumed that vectors are smaller...
                            if record.into_iter().all(|h| {*h!=interested}) {
                                record.push(interested);
                            }
                        } else {
                            interest_store.insert(interesting, vec![interested]);
                        }
                    }
                    let wid = (&workers).into_iter().min_by_key(|w| w.pending_jobs).unwrap().id;
                    let worker = workers.get_mut(wid).unwrap();
                    match worker.job_box.send((interesting, player.clone())) {
                        Err(_e) => (),
//debug
//println!("cannot submit new job to {} ({}). worker has quit?", worker.id, _e),
//
                        Ok(_) => (),
                    };
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
        game_hash:GameHash,
        p:&Player,
    ) -> (GameState,Vec<GameHash>) {
        let mut cf = game_from_hash(game_hash);

        let options = cf.possible_moves(p);
        if options.is_empty() { // no possible moves left: stalemate
            return (GameState::Decided(Score::Remis(0), None), vec![]);
        }

        let mut draw_moves = Vec::<(Score,Column)>::new();
        let mut doomed_moves = Vec::<(Score,Column)>::new();
        let mut open_moves = Vec::<GameHash>::new();

        for mv in options.into_iter() {
            let score = cf.make_shading_move(p, Rc::clone(&mv));
            match score {
                Ok(score) => match score {
                    // found a winning move: immediate return
                    Score::Won(in_n) => {
                        cf.withdraw_move(p, Rc::clone(&mv));
                        return (GameState::Decided(Score::Won(in_n+1), Some(mv.data().clone())), vec![]);
                    },
                    // found an undecided move, winning is still an option: let opponent make a move
                    Score::Undecided(_) => {
                        let anti_options = cf.possible_moves(p.opponent());
                        if anti_options.is_empty() { // no possible moves left: stalemate
                            draw_moves.push((Score::Remis(1), mv.data().clone())); ;
                        } else {
                            let mut anti_draw_moves = Vec::<(Score,Column)>::new();
                            let mut anti_doomed_moves = Vec::<(Score,Column)>::new();
                            let mut anti_open_moves = Vec::<GameHash>::new();
                            let mut anti_won = false;

                            for anti_mv in anti_options.into_iter() {
                                let anti_score = cf.make_shading_move(p.opponent(), Rc::clone(&anti_mv));
                                match anti_score {
                                    Ok(score) => match score {
                                        Score::Won(in_n) => { // opponent has a winning move: losing
                                            doomed_moves.push((Score::Lost(in_n+2), mv.data().clone()));
                                            cf.withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                                            anti_won = true;
                                            break;
                                        },
                                        Score::Lost(in_n) => { anti_doomed_moves.push((Score::Lost(in_n+1), mv.data().clone())); },
                                        Score::Remis(in_n) => { anti_draw_moves.push((Score::Remis(in_n+1), mv.data().clone())); },
                                        Score::Undecided(_) => { // unclear from the bord: check game store
                                            let hash = hash_from_state(cf.state());
//print!("+.");
                                            let gs = game_store.lock().unwrap();
//print!(".+");
                                            if let Some(record) = (*gs).get(&hash) {
                                                match &record.state {
                                                    GameState::Decided(record_score,_) => match record_score {
                                                        Score::Lost(in_n) => { // opponent can reach a lost game: losing
                                                            doomed_moves.push((Score::Lost(in_n+2), mv.data().clone()));
                                                            cf.withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                                                            anti_won = true;
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
                                cf.withdraw_move(p.opponent(), Rc::clone(&anti_mv));
                            }

                            if anti_won {
                                // doomed_moves.push(anti_won_move);
                            } else if !anti_open_moves.is_empty() { // best opponent move is undecided
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
                                    cf.withdraw_move(p, Rc::clone(&mv));
                                    return (GameState::Decided(Score::Won(in_n+1), Some(mv.data().clone())), vec![]);
                                }
                            }

                        }
                    },
                    Score::Remis(in_n) => { draw_moves.push((Score::Remis(in_n+1), mv.data().clone())); },
                    Score::Lost(in_n) => { doomed_moves.push((Score::Lost(in_n+1), mv.data().clone())); },
                },
                Err(_) => panic!("unexpected error in move"),
            }
            cf.withdraw_move(p, Rc::clone(&mv));
        }

        // if there is a winning move, it was returned already
        if !open_moves.is_empty() { // best move is yet undecided
            return (GameState::Undecided, open_moves);
        } else if !draw_moves.is_empty() { // best move is a draw
            let (score, col) = draw_moves.first().unwrap();
            return (GameState::Decided(score.clone(), Some(col.clone())), vec![]);
        } else if !doomed_moves.is_empty() { // all is lost
            let (score, col) = doomed_moves.first().unwrap();
            return (GameState::Decided(score.clone(), Some(col.clone())), vec![]);
        }
        (GameState::Undecided, vec![])
    }

    fn game_simulation(
        moves_ahead:i32,
        g:Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>,
        p:&Player
    ) -> GameState {
        let cfs = ConnectFourStrategy::default();
// debug
//println!("{}\n{}\n{}", g.borrow().display(), moves_ahead, p);
//
        match cfs.find_best_move(g,p,moves_ahead,true) {
            (Some(mv), Some(score)) => match score {
                Score::Undecided(_) => GameState::Undecided,
                score => GameState::Decided(score, Some(mv.data().clone())),
            },
            (_,_) => GameState::Undecided,
        }
    }

    fn claim_interests(
        interest_sender:&Sender<Interest>,
        interesting_hashes:Vec<GameHash>,
        parent_hash:GameHash,
    ) {
        interesting_hashes.into_iter()
        .map(|ih| {
            Interest { 
                interested: Some(parent_hash),
                interesting: Some(ih),
                worker_id: None,
            }
        }).map(|im| {
            match interest_sender.send(im) {
                Err(_e) => (),
// debug
//println!("cannot send interest ({}), conductor has left the building?", _e),
//
                Ok(_) => (),
            }
        }).for_each(drop);
    }

    fn do_the_job(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            moves_ahead:i32,
            interest:&Sender<Interest>,
            hash:GameHash,
            p:&Player) {
        match Worker::lock_hash(&game_store, hash) {
            // 0. quit job if game is locked or decided
            Err(_) => return (),
            Ok(new) => { // new game, never simulated
                let game = Rc::new(RefCell::new(game_from_hash(hash)));

                if new { // new game, never simulated
            // 1. try to find a solution from game simulation - if not already tried!
                // TODO: skip simulation if already tried
                    match Worker::game_simulation(moves_ahead, game.clone(), p) {
                        GameState::Decided(verdict, mv) => { 
                            return Worker::unlock_hash(&game_store, hash, GameState::Decided(verdict, mv));
                        },
                        GameState::Locked => panic!("unexpected state at this state"),
                        GameState::Undecided => (),
                    }
                }

            // 2. try to find a solution from the game store two moves ahead
                match Worker::two_moves_ahead_inquiry(&game_store, hash, p) {
                    (GameState::Decided(verdict, mv),_) => { 
                        return Worker::unlock_hash(&game_store, hash, GameState::Decided(verdict, mv));
                    },
                    (GameState::Locked,_) => panic!("unexpected state at this state"),
                    (GameState::Undecided, interesting_hashes) => {
            // 3. claim interest for the remaining undecided games two moves ahead
                        // TODO: if necessary, defer the interesting hashes from the game simulation and not the two moves inquiry
                        //       should be more efficient!
                        Worker::claim_interests(interest, interesting_hashes, hash);
                        return Worker::unlock_hash(&game_store, hash, GameState::Undecided);
                    },
                }
            },
        }
    }
    fn lock_hash(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            hash:GameHash) -> Result<bool,bool> {
//print!("+-");
        let mut gs = game_store.lock().unwrap();
//print!("-+");
        if let Some(record) = (*gs).get(&hash) {
            match &record.state {
                GameState::Undecided => {
                    (*gs).insert(hash, GameRecord { state: GameState::Locked, });
// debug
//println!("locked {}", hash);
//
                    return Ok(false);
                },
                GameState::Locked => {
// debug
//println!("cannot lock {}, it's locked", hash);
//
                    return Err(true);
                },
                GameState::Decided(_score, _mv) => {
// debug
//println!("cannot lock {}, it's {:?} with {:?}", hash, score, mv);
//
                    return Err(false);
                },
            }
        } else {
// debug
//println!("new record {}", hash);
//
            (*gs).insert(hash, GameRecord { state: GameState::Locked, });
            return Ok(true);
        }
    }
    fn unlock_hash(
            game_store:&Arc<Mutex<HashMap<GameHash,GameRecord>>>,
            hash:GameHash,
            state:GameState) {
//print!("+-");
        let mut gs = game_store.lock().unwrap();
//print!("-+");
        match state {
            GameState::Locked => panic!("must unlock {}", hash),
            _ => (),
        }
        if let Some(record) = (*gs).get(&hash) {
            match record.state {
                GameState::Locked => {
                    (*gs).insert(hash, GameRecord{ state: state, });
                },
                _ => panic!("{} should be locked!", hash),
            }
        } else {
            panic!("{} should have a record!", hash);
        }
    }

    fn spawn_worker(
            wid:usize,
            interest:Sender<Interest>,
            moves_ahead:i32,
            game_store:Arc<Mutex<HashMap<GameHash,GameRecord>>>) -> Worker {
        let (tx,jobs) = channel::<(GameHash,Player)>();
        let moves_ahead = moves_ahead;
// debug
//println!("hello {}", wid);
//

thread::spawn(move|| {
    loop {
        match jobs.recv() {
            Err(e) => { println!("Job receive error - {}", e); }
            Ok((-1,_)) => {
// debug
//println!("bye-bye {}", wid);
//
                break;
            },
            Ok((hash,p)) => {
// debug
//println!("job for {}: {}", wid, hash);
//
                Worker::do_the_job(&game_store, moves_ahead, &interest, hash, &p);
                match interest.send(Interest{
                    interested: Some(hash), interesting: None, worker_id: Some(wid),
                }) {
                    Err(_e) => (),
//debug
//println!("cannot declare job done ({}). conductor has left the building?", _e),
//
                    Ok(_) => (),
                };

// debug
//println!("done by {}: {}", wid, hash);
//
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

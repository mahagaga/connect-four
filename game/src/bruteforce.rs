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

type GameHash = i64;

pub struct GameRecord {
    state: GameState,
}
pub enum GameState {
    Locked,
    Decided(Score),
    Undecided,
}
pub struct BruteForceStrategy {
    pub nworkers: i32,
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
        
        let (sender,receiver) = self.init_conductor_and_band();
        self.claim_public_interest(sender, g);
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

    fn init_conductor_and_band(&self) -> (Sender<Interest>, Receiver<Verdict>) {
        let (itx, interests) = channel::<Interest>();
        let (final_verdict, rx) = channel::<Verdict>();
        
        let game_store = Arc::new(Mutex::new(HashMap::<GameHash,GameRecord>::new()));

        thread::spawn(move|| {
            let mut interest_store:HashMap<GameHash,Vec<GameHash>> = HashMap::new();
            let mut workers:Vec<Worker> = Vec::new();
            loop {
                if let Ok(interest) = interests.recv() {
                    // worker has finished job
                    if let None = interest.interesting {
                        // note changed job pendencies
                        let worker_id = interest.worker_id.unwrap();
                        workers.get_mut(worker_id).unwrap().pending_jobs -= 1;
                        // submit new jobs if this game is now decided
                        let finished = interest.interested.unwrap();
                        let gs = game_store.lock().unwrap();
                        match (*gs).get(&finished) {
                            Some(record) => {
                                match &record.state {
                                    GameState::Decided(_) => (),
                                    GameState::Locked => panic!("shouldn't happen!?"),
                                    GameState::Undecided => (),
                                }
                            },
                            None => panic!("is not possibly possible!"),
                        }
                        
                    } else { // submit new job (unless it's the final verdict)

                    }         
                }
            }
        });
        (itx, rx)
    

    }

    fn claim_public_interest(&self,
            sender: Sender<Interest>,
            g: Rc<RefCell<dyn Game<Column,Vec<Vec<Option<Player>>>>>>
        ) {
        
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
    column: Column,
}

struct Interest {
    interested: Option<GameHash>,
    interesting: Option<GameHash>,
    worker_id: Option<usize>,
}


pub struct Worker {
    pending_jobs: u128,
    job_box: Sender<GameHash>,
}
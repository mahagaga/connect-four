//pub mod generic;
use generic::*;
use connectfour::*;
use std::rc::Rc;
use std::cell::RefCell;


//#################################################################################################
// specifically Connect Four
//#################################################################################################


//### connect four strategy #######################################################################

pub struct BruteForceStrategy {
    pub oscore_koeff: f32,
    pub mscore_koeff: f32,
    pub nscore_koeff: f32,
    pub my_tabu_koeff: f32,
    pub opp_tabu_koeff: f32,
    pub tabu_defense_koeff: f32,
}

enum Cell {
    M, //my stone
    O, //opponent's stone
    N, //no stone, empty cell
    D, //dead cell, game will be over before it is occupied
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

use std::cmp;
impl Strategy<Column,Vec<Vec<Option<Player>>>> for BruteForceStrategy {
    
    fn evaluate_move(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                     p: &Player, mv: Rc<Move<Column>>) 
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
        
        // identify dead cells
        let efield = self.fill_in_dead_cells(Rc::clone(&g), efield);

        // calculate score
        let total_score = self.positional_score(n, m, &efield)
                        + self.tabu_diff_score(g, p, mv);
        Ok(total_score)
    }
}

#[derive(Debug)]
struct Tabu {
    column: Column,
    mine: Option<u32>,
    theirs: Option<u32>,
}

impl BruteForceStrategy {
    pub fn default() -> Self {
        BruteForceStrategy { 
            mscore_koeff: 1.0,
            oscore_koeff: 0.8,
            nscore_koeff: 0.5,
            my_tabu_koeff: -10.0,
            opp_tabu_koeff: 10.0,
            tabu_defense_koeff: 0.25,
        }
    }

    fn fill_in_dead_cells(&self, 
            g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
            mut efield: Vec<Vec<Cell>>)  -> Vec<Vec<Cell>> {
        let mut mutable_game = g.borrow_mut();
        
        let mut cp = &Player::White;
        for col in 0..ConnectFour::width() {
            // look for mutual tabus
            let mut mv = Rc::new(ConnectFourMove {
                data: Column::from_usize(col) 
            });
            let mut i = 0;
            while let Ok(score) = mutable_game.make_move(cp, mv.clone()) {
                i += 1;
                if let Score::Won(_) = score {
                    mutable_game.withdraw_move(cp, mv.clone());
                    cp = cp.opponent();
                    // unwrap in the next line assumed to be save because of the preceding withdrawal
                    if let Score::Won(_) = mutable_game.make_move(cp, mv.clone()).unwrap() {
                        for j in mutable_game.state()[col].len()..ConnectFour::height() {
                            efield[col][j] = Cell::D;
                        }
                        break;
                    }
                    
                }
                cp = cp.opponent();
            }
            //println!("{}", mutable_game.display());
            for _ in 0..i {
                cp = cp.opponent();
                mutable_game.withdraw_move(cp, mv.clone());
            }
        }
        efield
    }

    // comparing tabu rows before and after the move
    // panicks if the move is not allowed
    fn tabu_diff_score(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                  p: &Player, mv: Rc<Move<Column>>)  -> f32 {
        let ground_score = self.tabu_score(Rc::clone(&g), p);

        g.borrow_mut().make_move(p, Rc::clone(&mv)).unwrap();
        let offense_score = self.tabu_score(Rc::clone(&g), p) - ground_score;     
        g.borrow_mut().withdraw_move(p, Rc::clone(&mv));

        g.borrow_mut().make_move(p.opponent(), Rc::clone(&mv)).unwrap();
        let defense_score = self.tabu_score(Rc::clone(&g), p) - ground_score;     
        g.borrow_mut().withdraw_move(p.opponent(), Rc::clone(&mv));
        
        offense_score - defense_score * self.tabu_defense_koeff
    }

    fn tabu_score(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                  p: &Player)  -> f32 {
        let mut mutable_game = g.borrow_mut();
        
        (0..ConnectFour::width()) // loop over columns
        .map(|col| {
            // look for tabus
            let mut tabu = Tabu{ column: Column::from_usize(col),
                                 mine: None, theirs: None, };
            let mut cp = p;
            let mut i = 0;
            while let Ok(score) = mutable_game.make_move(cp, Rc::new(
                    ConnectFourMove { data: Column::from_usize(col) })) {
                cp = cp.opponent();
                i += 1;
                match score {
                    Score::Won(_) => {
                        match i%2 {
                            0 => { tabu.mine = Some(i-1); },
                            _ => (),
                        }
                        //tabu.me_first = Some((match i%2 { 1 => Who::Them, _ => Who::Me, }, i-1));
                        break;
                    },
                    _ => (),
                }
            }
            //println!("{}", mutable_game.display());
            for _ in 0..i {
                cp = cp.opponent();
                mutable_game.withdraw_move(cp, Rc::new(
                    ConnectFourMove { data: Column::from_usize(col) }));
            }

            let mut cp = p.opponent();
            let mut i = 0;
            while let Ok(score) = mutable_game.make_move(cp, Rc::new(
                    ConnectFourMove { data: Column::from_usize(col) })) {
                cp = cp.opponent();
                i += 1;
                match score {
                    Score::Won(_) => { 
                        match i%2 {
                            0 => { tabu.theirs = Some(i-1); },
                            _ => (),
                        }
                        //tabu.opp_first = Some((match i%2 { 1 => Who::Me, _ => Who::Them, }, i-1));
                        break;
                    },
                    _ => (),
                }
            }
            //println!("{}", mutable_game.display());
            for _ in 0..i {
                cp = cp.opponent();
                mutable_game.withdraw_move(cp, Rc::new(
                    ConnectFourMove { data: Column::from_usize(col) }));
            }
            
            if let Some(_) = tabu.mine {
                //println!("{:?}", &tabu);
            } else if let Some(_) = tabu.theirs {
                //println!("{:?}", &tabu);
            }
            tabu
        })
        .map(|tabu| {
            let mine = match tabu.mine {
                Some(i) => self.my_tabu_koeff / i as f32,
                None => 0.0,
            };
            let theirs = match tabu.theirs {
                Some(i) => self.opp_tabu_koeff / i as f32,
                None => 0.0,
            };
            mine + theirs
        })
        .sum()
    }
    
    // basically adding up the user's own potential for connecting four from/to 
    // here and the opponents, weighed by the strategy's coefficients
    fn positional_score(&self, n:usize, m:usize, efield:&Vec<Vec<Cell>>) -> f32 {
        let score_arithmetics = |((mfree_left, m_left, nm_left), (ofree_left, o_left, no_left)), 
                                ((mfree_right,m_right,nm_right),(ofree_right,o_right,no_right))| -> f32 {
            let mut partial_score = 0.0;
            if mfree_left + mfree_right >= 3 {
                partial_score += self.mscore_koeff * (m_left + m_right) as f32;
                partial_score += self.mscore_koeff * self.nscore_koeff * (nm_left + nm_right) as f32;
            }
            if ofree_left + ofree_right >= 3 {
                partial_score += self.oscore_koeff * (o_left + o_right) as f32;
                partial_score += self.oscore_koeff * self.nscore_koeff * (no_left + no_right) as f32;
            }
            partial_score
        };

        let mut total_score = 0.0;
        // horizontal score
        let ontheleft = self.efield_counting(efield,
            (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect(),
            vec!(m,m,m));
        let ontheright = self.efield_counting(efield,
            (cmp::min(ConnectFour::width(), n+1)..cmp::min(ConnectFour::width(), n+4)).collect(),
            vec!(m,m,m));
        total_score += score_arithmetics(ontheleft, ontheright);

        // diagonal score '/'
        let ontheleft = self.efield_counting(efield,
            (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect(),
            (match m { s if s < 3 => 0, b => b-3 }..m).rev().collect());
        let ontheright = self.efield_counting(efield,
            (cmp::min(ConnectFour::width(), n+1)..cmp::min(ConnectFour::width(), n+4)).collect(),
            (cmp::min(ConnectFour::height(),m+1)..cmp::min(ConnectFour::height(),m+4)).collect());
        total_score += score_arithmetics(ontheleft, ontheright);

        // diagonal score '\'
        let ontheleft = self.efield_counting(efield,
            (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect(),
            (cmp::min(ConnectFour::height(),m+1)..cmp::min(ConnectFour::height(),m+4)).collect());
        let ontheright = self.efield_counting(efield,
            (cmp::min(ConnectFour::width(), n+1)..cmp::min(ConnectFour::width(), n+4)).collect(),
            (match m { s if s < 3 => 0, b => b-3 }..m).rev().collect());
        total_score += score_arithmetics(ontheleft, ontheright);

        // vertical score
        let ontheleft = self.efield_counting(efield,
            vec!(n,n,n),
            (match m { s if s < 3 => 0, b => b-3 }..m).rev().collect());
        let ontheright = self.efield_counting(efield,
            vec!(n,n,n),
            (cmp::min(ConnectFour::height(), m+1)..cmp::min(ConnectFour::height(), m+4)).collect());
        total_score += score_arithmetics(ontheleft, ontheright);

        total_score
    }
}
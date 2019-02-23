//pub mod generic;
use generic::*;
use std::rc::Rc;
use std::cell::RefCell;


//#################################################################################################
// specifically Connect Four
//#################################################################################################

pub struct ConnectFour {
    field: Vec<Vec<Option<Player>>>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Column {
    One, Two, Three, Four, Five, Six, Seven, Zero
}

impl Column {
    pub fn to_usize(&self) -> usize {
        match &self {
            Column::One => 0x0,
            Column::Two => 0x1,
            Column::Three => 0x2,
            Column::Four => 0x3,
            Column::Five => 0x4,
            Column::Six => 0x5,
            Column::Seven => 0x6,
            // Zero is for making the reverse function from_usize easier to use
            Column::Zero => 0x99,
        }
    }

    pub fn from_usize(i: usize) -> Self {
        match i {
            0x0 => Column::One,
            0x1 => Column::Two,
            0x2 => Column::Three,
            0x3 => Column::Four,
            0x4 => Column::Five,
            0x5 => Column::Six,
            0x6 => Column::Seven,
            _ => Column::Zero,
        }
    }
}

pub struct ConnectFourMove {
    pub data: Column,
}

impl Move<Column> for ConnectFourMove {
    fn data(&self) -> &Column {
        &self.data
    }

    fn display(&self) -> String {
        let s = format!("{:?}", self.data());
        s
    }
}

impl Game<Column,Vec<Vec<Option<Player>>>> for ConnectFour {
    fn possible_moves(&self, _: &Player) -> Vec<Rc<Move<Column>>> {
        let mut allowed: Vec<Rc<Move<Column>>> = Vec::new();
        let mut i:usize = 0;
        for col in &self.field {
            if col.len() < ConnectFour::height() {
                allowed.push(Rc::new(ConnectFourMove {
                    data: Column::from_usize(i)
                }));
            }
            i += 1;
        }
        allowed
    }

    fn make_move(&mut self, p: &Player, mv: Rc<Move<Column>>) -> Result<Score, Withdraw> {
        let n = mv.data().to_usize();
        if ConnectFour::height() == self.field[n].len() {
            // column is obviously already filled to the top
            Err(Withdraw::NotAllowed)
        } else {
            self.field[n].push(match p {
                Player::White => Some(Player::White),
                Player::Black => Some(Player::Black),
                Player::Grey => Some(Player::Grey),
            });
            self.get_score(p, n, self.field[n].len()-1)
        }
    }

    fn withdraw_move(&mut self, _p: &Player, mv: Rc<Move<Column>>) {
        let n = mv.data().to_usize();
        self.field[n].pop();
    }

    fn display(&self) -> String {
        let mut s = String::new();
        s.push_str("------\n");
        for c in &self.field {
            for x in c {
                match x {
                    Some(p) => match p { Player::White => { s.push_str("o"); },
                                         Player::Black => { s.push_str("x"); },
                                         Player::Grey => { s.push_str("n")}
                    },
                    None => (),
                }
            }
            s.push_str("\n");
        }
        s.push_str("------");
        s
    }

    fn state(&self) -> &Vec<Vec<Option<Player>>> {
        &self.field
    }
}

enum Step {
    Up,
    Down,
    Plane,
}

//### connect four ################################################################################

impl ConnectFour {
    pub fn width() -> usize { 7 }
    pub fn height() -> usize { 6 }
    
    pub fn new() -> Self {
        let mut cf = ConnectFour{
            field: Vec::with_capacity(ConnectFour::width()),
        };
        for _coln in 0..ConnectFour::width() {
            let mut col:Vec<Option<Player>> = Vec::with_capacity(ConnectFour::height());
            cf.field.push(col);
        };
        cf
    }

    pub fn clone(&self) -> ConnectFour {
        let mut cf = ConnectFour{
            field: Vec::with_capacity(ConnectFour::width()),
        };
        for self_col in &self.field {
            let mut col:Vec<Option<Player>> = Vec::with_capacity(ConnectFour::height());
            for player_option in self_col {
                col.push(match player_option {
                    Some(player) => match player {
                          Player::White => Some(Player::White),
                          Player::Black => Some(Player::Black),
                          Player::Grey => Some(Player::Grey),
                    }, 
                    None => None, 
                });
            }
            cf.field.push(col);
        };
        cf
    }

    fn get_score(&self, p: &Player, n: usize, m: usize) -> Result<Score, Withdraw> {

        // vertical
        let below = self.matching_distance(vec![n,n,n], m, Step::Down, p);
        if below >= 3 {
            //println!("{} below {}", below, m);
            return Ok(Score::Won(0))
        }

        // horizontal
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Plane, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Plane, p);
        if left + right >= 3 {
            //println!("left {}, right {}", left, right);
            return Ok(Score::Won(0))
        }

        // diagonal (\)
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Up, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Down, p);
        if left + right >= 3 {
            //println!("\\left {}, right {}", left, right);
            return Ok(Score::Won(0))
        }

        // diagonal (/)
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Down, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Up, p);
        if left + right >= 3 {
            //println!("/left {}, right {}", left, right);
            return Ok(Score::Won(0))
        }

        // last stone
        if !self.move_possible() {
            return Ok(Score::Remis(0))
        }

        Ok(Score::Undecided(0.5))
    }

    fn matching_distance(&self, 
            iter: Vec<usize>, 
            m: usize,
            step: Step,
            p: &Player) -> usize {
        let mut distance = 1;
        for i in iter.into_iter() {
            let j:usize = match step {
                Step::Up => m+distance,
                Step::Down => { if distance>m {
                                return distance-1; 
                            }
                            m-distance },
                Step::Plane => m,
            };
            
            if j>=self.field[i].len() {
                return distance-1
            }
            match &self.field[i][j] {
                Some(cp) => {
                    if *cp == *p {
                        //println!("{} {} matches, dist {} up", i, j, distance);
                        distance += 1;
                    } else {
                        break;
                    }
                },
                None => {
                    break;
                }
            }
        }
        distance-1
    }

    pub fn drop_stone(&mut self, p: &Player, c:Column) -> Result<Score, Withdraw> {
        self.make_move(&p, Rc::new(ConnectFourMove { data: c }))
    }

    pub fn undrop_stone(&mut self, p: &Player, c:Column) {
        self.withdraw_move(&p, Rc::new(ConnectFourMove { data: c }))
    }

    fn move_possible(&self) -> bool {
        for col in &self.field {
            if col.len() < ConnectFour::height() {
                return true;
            }
        }
        false
    }

    pub fn replicate(plan: String) -> ConnectFour {
        let mut g = ConnectFour::new();
        for (i, line) in plan.split("\n").enumerate() {
            match i {
                b if (b > 0 && b < 8) => {
                    for c in line.chars() {
                        g.drop_stone(
                            match c {
                                'x' => &Player::Black,
                                'o' => &Player::White,
                                'n' => &Player::Grey,
                                what => { println!("{}, {}", what, i); panic!(); },
                            },
                            Column::from_usize(i-1)
                        ).unwrap(); 
                    }
                },
                _ => assert_eq!(line, "------"),
            }
        }
        g
    }
}

//### connect four strategy #######################################################################
pub struct NaiveStrategy {}
impl Strategy<Column,Vec<Vec<Option<Player>>>> for NaiveStrategy {
    fn evaluate_move(&self, _g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>,
                     _p: &Player, _mv: Rc<Move<Column>>) -> Result<f32,Withdraw> {
        Ok(0.0)
    }
}

pub struct ConnectFourStrategy {
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

impl ConnectFourStrategy {
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
impl Strategy<Column,Vec<Vec<Option<Player>>>> for ConnectFourStrategy {
    
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
        let black = |player: &Player| { match player { Player::Black => Cell::M, Player::White => Cell::O, Player::Grey => Cell::D, }};
        let white = |player: &Player| { match player { Player::White => Cell::M, Player::Black => Cell::O, Player::Grey => Cell::D, }};
        let mut i:usize = 0;
        for c in g.borrow().state() { // that's the current Connect Four field
            let mut j:usize = 0;
            for f in c {
                match f {
                    Some(Player::Black) => efield[i][j] = black(p),
                    Some(Player::White) => efield[i][j] = white(p),
                    Some(Player::Grey) => efield[i][j] = Cell::D,
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

impl ConnectFourStrategy {
    pub fn default() -> Self {
        ConnectFourStrategy { 
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
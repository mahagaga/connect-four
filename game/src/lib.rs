//#################################################################################################
// tests
//#################################################################################################

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_column() {
        assert_eq!(Column::Five.to_usize(), 0x4);
    }
    #[test]
    fn test_move() {
        let white = Player::White;
            
        let middle = Box::new(ConnectFourMove { data: Column::Four });
        assert_eq!(middle.data().to_usize(), 0x3);

        // drop 7 white Stones in the middle column
        let mut cf = ConnectFour::new();
        for i in 0..7 {
            //println!("drop {} time", i+1);
            let middle = Rc::new(ConnectFourMove { data: Column::Four });
            match cf.make_move(&white, middle) {
                Ok(x) => match x {
                    // should be undecided 3 times
                    Score::Undecided(_p) => assert!(i<3,i),
                    // then won 3 times
                    Score::Won => assert!(i>2,i),
                    _ => assert!(false),
                }
                // the 7th stone is one too many
                _ => assert!(i>5),
            }
        }

        // drop 4 stones in a row
        let mut cf = ConnectFour::new();
        match cf.make_move(&white, Rc::new(ConnectFourMove { data: Column::Four })) {
            Ok(x) => if let Score::Undecided(_p) = x { () } else { assert!(false)},
            _ => assert!(false),
        }
        match cf.make_move(&white, Rc::new(ConnectFourMove { data: Column::Two })) {
            Ok(x) => if let Score::Undecided(_p) = x { () } else { assert!(false)},
            _ => assert!(false),
        }
        match cf.make_move(&white, Rc::new(ConnectFourMove { data: Column::Five })) {
            Ok(x) => if let Score::Undecided(_p) = x { () } else { assert!(false)},
            _ => assert!(false),
        }
        match cf.make_move(&white, Rc::new(ConnectFourMove { data: Column::Three })) {
            Ok(x) => if let Score::Won = x { () } else { assert!(false)},
            _ => assert!(false),
        }
    }

    #[test]
    fn test_possible_moves() {
        let mut cf = ConnectFour::new();
        let p = Player::Black;
        let pm = cf.possible_moves(&p);
        assert!(pm.len()==ConnectFour::width());
        assert!(*pm[3].data() == Column::Four);

        for _ in 0..6 {
            let _ = cf.drop_stone(&p, Column::Four);
        }
        let pm:Vec<Rc<Move<Column>>> = cf.possible_moves(&p);
        println!("{:?}", &cf.field);
        for x in &pm {
            println!("{:?}", &x.data());
        }
        assert!(pm.len()==ConnectFour::width()-1);
        assert!(*pm[3].data() == Column::Five);
    }

    #[test]
    fn test_find_best_move() {
        let white = Player::White;
        let black = Player::Black;
        let strategy = ConnectFourStrategy {
             mscore_koeff: 1.0,
             oscore_koeff: 0.8,
             nscore_koeff: 0.5
        };

        // recognize a winner
        let mut game = ConnectFour::new();
        for _ in 0..3 { 
            let score = game.drop_stone(&white, Column::Six); 
            match score {
                Ok(Score::Undecided(p)) => assert!(p == 0.5),
                _ => assert!(false),
            };
        }
        if let (Some(mv), Some(score)) = strategy.find_best_move(
                Rc::new(RefCell::new(game)), &white, 0, false) {
            println!("{:?} {:?}", mv.data(), score);
            assert!(Score::Won == score);
            assert!(*mv.data() == Column::Six);
        } else { assert!(false); }

        // danger awareness
        game = ConnectFour::new();
        for _ in 0..3 { 
            let score = game.drop_stone(&black, Column::Six); 
            match score {
                Ok(Score::Undecided(p)) => assert!(p == 0.5),
                _ => assert!(false),
            };
        }
        if let (Some(mv), Some(score)) = strategy.find_best_move(
                Rc::new(RefCell::new(game)), &white, 1, false) {
            println!("{:?} {:?}", mv.data(), score);
            match score {
                Score::Undecided(p) => assert!(p == 0.5),
                _ => { println!("didn't catch the danger"); assert!(false); },
            }
            assert!(*mv.data() == Column::Six);
        } else { assert!(false); }

        // first move
        game = ConnectFour::new();
        if let (Some(mv), Some(score)) = strategy.find_best_move(
                Rc::new(RefCell::new(game)), &white, 2, true) {
            println!("{:?} {:?} vs {}", mv.data(), score,
                              12 as f32 * strategy.mscore_koeff * strategy.nscore_koeff
                            + 12 as f32 * strategy.oscore_koeff * strategy.nscore_koeff);
            match score {
                    Score::Undecided(ev) => assert!(
                        ev == 12 as f32 * strategy.mscore_koeff * strategy.nscore_koeff
                            + 12 as f32 * strategy.oscore_koeff * strategy.nscore_koeff),
                    _ => assert!(false),
            }
            assert!(*mv.data() == Column::Four);
        } else { assert!(false); }
    }

    #[test]
    fn test_evaluate_move() {
        let mut g = ConnectFour::new();
        let black = Player::Black;
        let white = Player::White;
        let _ = g.drop_stone(&black, Column::Two);
        let mv = ConnectFourMove { data: Column::Five, };
        let s = ConnectFourStrategy  {
             mscore_koeff: 1.0,
             oscore_koeff: 0.8,
             nscore_koeff: 0.5,
        };

        let expected = 7 as f32 * s.mscore_koeff * s.nscore_koeff
                     + 0 as f32 * s.mscore_koeff
                     + 7 as f32 * s.oscore_koeff * s.nscore_koeff
                     + 1 as f32 * s.oscore_koeff;
        if let Ok(eval) = s.evaluate_move(Rc::new(RefCell::new(g)), &white, Rc::new(mv)) {
            println!("expected score {} vs calculated {}", expected, eval);
            assert!(eval == expected)
        } else { assert!(false) }
    }
}

//#################################################################################################
// generic game with two players
//#################################################################################################

use std::rc::Rc;
use std::cell::RefCell;

#[derive(PartialEq, Eq, Debug)]
pub enum Player {
    Black,
    White,
}

impl Player {
    fn opponent(&self) -> &Player {
        match self {
            Player::Black => &Player::White,
            Player::White => &Player::Black,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Player::Black => write!(f, "{}", String::from("Black")),
            Player::White => write!(f, "{}", String::from("White")),
        }
    }
}

pub trait Move<T> {
    fn data(&self) -> &T;
    fn display(&self) -> String;
}

#[derive(PartialEq, Debug)]
pub enum Score {
    Undecided(f32),
    Remis,
    Won,
    Lost,
}

#[derive(Debug)]
pub enum Withdraw {
    NotAllowed,
}

pub trait Game<T,S> {
    fn possible_moves(&self, p: &Player) -> Vec<Rc<Move<T>>>;
    fn make_move(&mut self, p: &Player, m: Rc<Move<T>>) -> Result<Score, Withdraw>;
    fn withdraw_move(&mut self, p: &Player, m: Rc<Move<T>>);
    fn display(&self) -> String;
    fn state(&self) -> &S;
}

//### strategy ####################################################################################

pub trait Strategy<T,S> {
    fn evaluate_move(&self, g: Rc<RefCell<Game<T,S>>>, p: &Player, m: Rc<Move<T>>) -> Result<f32, Withdraw>;

    fn find_best_move(&self, 
            g: Rc<RefCell<Game<T,S>>>,
            p: &Player,
            moves_ahead: i32,
            game_evaluation: bool
        ) -> (Option<Rc<Move<T>>>, Option<Score>) {

        //let mut win_option: Option<Rc<Move<T>>> = None;
        let mut remis_option: Option<Rc<Move<T>>> = None;
        let mut lost_option: Option<Rc<Move<T>>> = None;
        let mut follow_ups: Vec<(Rc<Move<T>>, f32)> = Vec::new();
        
        let options = g.borrow().possible_moves(p);
        for mv in options.into_iter() {
            let score = g.borrow_mut().make_move(p, Rc::clone(&mv));
            match score {
                Ok(score) => match score {
                    Score::Won => {
                        //println!("{}", &g.borrow().display());
                        //println!("{:?} wins with {:?}", p, mv.display());
                        g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
                        return (Some(mv), Some(Score::Won));;
                    },
                    Score::Remis => { remis_option = Some(Rc::clone(&mv)); },
                    Score::Lost => { lost_option = Some(Rc::clone(&mv)); },
                    Score::Undecided(pv) => { follow_ups.push((Rc::clone(&mv), pv)); },
                },
                Err(_) => (),//return Err(_),
            }
            g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
        }
        
        let mut still_undecided: Vec<(Rc<Move<T>>, f32)> = Vec::new();
        for (undecided, pv) in follow_ups {
            if moves_ahead > 0 {
                let _ = g.borrow_mut().make_move(p, Rc::clone(&undecided));
                let (_, advscore) = self.find_best_move(Rc::clone(&g), p.opponent(), moves_ahead-1, false);
                match advscore {
                    Some(Score::Won) => { lost_option = Some(Rc::clone(&undecided)); },
                    Some(Score::Remis) => { remis_option = Some(Rc::clone(&undecided)); },
                    Some(Score::Lost) => { 
                        g.borrow_mut().withdraw_move(p, Rc::clone(&undecided));
                        return (Some(undecided), Some(Score::Won));
                    },
                    Some(Score::Undecided(advpv)) => {
                        still_undecided.push((Rc::clone(&undecided), 1.0-advpv));
                    },
                    None => println!("why is here None?"),
                }
                g.borrow_mut().withdraw_move(p, Rc::clone(&undecided));
            } else {
                still_undecided.push((Rc::clone(&undecided), pv));
            }
        }

        let mut undecided_option: Option<Rc<Move<T>>> = None;
        let mut undecided_pv = -1.0;
        for (undecided, pv) in still_undecided {
            if game_evaluation {
                match self.evaluate_move(Rc::clone(&g), p, Rc::clone(&undecided)) {
                    Ok(ev) => if ev > undecided_pv {
                        if ev > undecided_pv {
                            undecided_option = Some(undecided);
                            undecided_pv = ev;
                        }
                    },
                    Err(e) => println!("what's wrong with {:?}: {:?}", undecided.display(), e),
                }
                
            } else {
                if pv > undecided_pv {
                    undecided_option = Some(undecided);
                    undecided_pv = pv;
                }
            }
        }

        if let Some(undecided) = undecided_option {
            if let Some(remis) = remis_option {
                if undecided_pv >= 0.5 { return (Some(undecided), Some(Score::Undecided(undecided_pv))); }
                else { return (Some(remis), Some(Score::Remis)); }
            } else {
                return (Some(undecided), Some(Score::Undecided(undecided_pv)));
            }
        }

        if let Some(remis) = remis_option { return (Some(remis), Some(Score::Remis)); }
        if let Some(lost) = lost_option { return (Some(lost), Some(Score::Lost)); }
        (None, None)
    }
}

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
                    Some(p) => match p { Player::White => { s.push_str("x"); },
                                         Player::Black => { s.push_str("o"); },
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
    fn width() -> usize { 7 }
    fn height() -> usize { 6 }
    
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
            return Ok(Score::Won)
        }

        // horizontal
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Plane, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Plane, p);
        if left + right >= 3 {
            //println!("left {}, right {}", left, right);
            return Ok(Score::Won)
        }

        // diagonal (\)
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Up, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Down, p);
        if left + right >= 3 {
            //println!("\\left {}, right {}", left, right);
            return Ok(Score::Won)
        }
        // diagonal (/)
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Down, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Up, p);
        if left + right >= 3 {
            //println!("/left {}, right {}", left, right);
            return Ok(Score::Won)
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
}

pub struct ConnectFourStrategy {
    pub oscore_koeff: f32,
    pub mscore_koeff: f32,
    pub nscore_koeff: f32,
}

enum Cell {
    M, //my stone
    O, //opponent's stone
    N, //no stone, empty cell
}

impl ConnectFourStrategy {
    #[allow(dead_code)]
    fn display_efield(&self, ef: &Vec<Vec<Cell>>) {
        for j in (0..ConnectFour::height()).rev() {
            for i in 0..ConnectFour::width() {
                print!("{}", match ef[i][j] {
                    Cell::N => ".",
                    Cell::M => "x",
                    Cell::O => "o",
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
    
    fn evaluate_move(&self, g: Rc<RefCell<Game<Column,Vec<Vec<Option<Player>>>>>>, p: &Player, mv: Rc<Move<Column>>) 
    -> Result<f32, Withdraw> {
        let n = mv.data().to_usize();
        let m = g.borrow().state()[n].len();

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
        for c in  g.borrow().state() { // that's the current Connect Four field
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
        
        let mut total_score = 0.0;
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

        // horizontal score
        let show:Vec<usize> = (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect();
        println!("{:?} {} {}", show, match n { s if s < 3 => 0, b => b-3 }, n);
        let ontheleft = self.efield_counting(&efield,
            (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect(),
            vec!(m,m,m));
        let ontheright = self.efield_counting(&efield,
            (cmp::min(ConnectFour::width(), n+1)..cmp::min(ConnectFour::width(), n+4)).collect(),
            vec!(m,m,m));
        total_score += score_arithmetics(ontheleft, ontheright);

        // diagonal score '/'
        let ontheleft = self.efield_counting(&efield,
            (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect(),
            (match m { s if s < 3 => 0, b => b-3 }..m).rev().collect());
        let ontheright = self.efield_counting(&efield,
            (cmp::min(ConnectFour::width(), n+1)..cmp::min(ConnectFour::width(), n+4)).collect(),
            (cmp::min(ConnectFour::height(),m+1)..cmp::min(ConnectFour::height(),m+4)).collect());
        total_score += score_arithmetics(ontheleft, ontheright);

        // diagonal score '\'
        let ontheleft = self.efield_counting(&efield,
            (match n { s if s < 3 => 0, b => b-3 }..n).rev().collect(),
            (cmp::min(ConnectFour::height(),m+1)..cmp::min(ConnectFour::height(),m+4)).collect());
        let ontheright = self.efield_counting(&efield,
            (cmp::min(ConnectFour::width(), n+1)..cmp::min(ConnectFour::width(), n+4)).collect(),
            (match m { s if s < 3 => 0, b => b-3 }..m).rev().collect());
        total_score += score_arithmetics(ontheleft, ontheright);

        // vertical score
        let ((_, m_below, _),(_, o_below, _)) = self.efield_counting(&efield,
            vec!(n,n,n),
            (match m { s if s < 3 => 0, b => b-3 }..m).rev().collect());
        total_score += self.mscore_koeff * m_below as f32;
        total_score += self.oscore_koeff * o_below as f32;
        
        Ok(total_score)
    }
}

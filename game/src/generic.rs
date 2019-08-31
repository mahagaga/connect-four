//#################################################################################################
// generic game with two players
//#################################################################################################

use std::rc::Rc;
use std::cell::RefCell;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Player {
    Black,
    White,
    Gray,
}

impl Player {
    pub fn opponent(&self) -> &Player {
        match self {
            Player::Black => &Player::White,
            Player::White => &Player::Black,
            Player::Gray => &Player::Gray,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Player::Black => write!(f, "{}", String::from("Black")),
            Player::White => write!(f, "{}", String::from("White")),
            Player::Gray => write!(f, "{}", String::from("Gray")),
        }
    }
}

pub trait Move<T> {
    fn data(&self) -> &T;
    fn display(&self) -> String;
}

#[derive(PartialEq, Debug, Clone)]
pub enum Score {
    Undecided(f32),
    Remis(u32),
    Won(u32),
    Lost(u32),
}

#[derive(Debug)]
pub enum Withdraw {
    NotAllowed,
}

pub trait Game<T,S> {
    fn possible_moves(&self, p: &Player) -> Vec<Rc<dyn Move<T>>>;
    fn make_move(&mut self, p: &Player, m: Rc<dyn Move<T>>) -> Result<Score, Withdraw>;
    fn withdraw_move(&mut self, p: &Player, m: Rc<dyn Move<T>>);
    fn display(&self) -> String;
    fn state(&self) -> &S;
}

//### strategy ####################################################################################

pub trait Strategy<T,S> {
    fn evaluate_move(&self, g: Rc<RefCell<dyn Game<T,S>>>, p: &Player, m: Rc<dyn Move<T>>) -> Result<f32, Withdraw>;

    fn find_best_move(&self, 
            g: Rc<RefCell<dyn Game<T,S>>>,
            p: &Player,
            moves_ahead: i32,
            game_evaluation: bool
        ) -> (Option<Rc<dyn Move<T>>>, Option<Score>) {

        //let mut win_option: Option<Rc<Move<T>>> = None;
        let mut remis_option: Option<(Rc<dyn Move<T>>,u32)> = None;
        let mut lost_options: Vec<(Rc<dyn Move<T>>,u32)> = Vec::new();
        let mut undecided_options: Vec<(Rc<dyn Move<T>>, f32)> = Vec::new();
        
        let options = g.borrow().possible_moves(p);
        for mv in options.into_iter() {
            let score = g.borrow_mut().make_move(p, Rc::clone(&mv));
                        
            match score {
                Ok(score) => match score {
                    Score::Won(in_n) => {
                        //println!("{}", &g.borrow().display());
                        //println!("{:?} wins with {:?} in {}", p, mv.display(), in_n);
                        g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
                        return (Some(mv), Some(Score::Won(in_n)));
                    },
                    Score::Remis(in_n) => { remis_option = Some((Rc::clone(&mv), in_n)); },
                    Score::Lost(in_n) => { lost_options.push((Rc::clone(&mv), in_n)); },
                    Score::Undecided(pv) => { undecided_options.push((Rc::clone(&mv), pv)); },
                },
                Err(_) => (),//return Err(_),
            }
            g.borrow_mut().withdraw_move(p, Rc::clone(&mv));
        }
        
        let mut still_undecided: Vec<(Rc<dyn Move<T>>, f32)> = Vec::new();
        for (undecided, pv) in undecided_options {
            if moves_ahead > 0 {
                let _ = g.borrow_mut().make_move(p, Rc::clone(&undecided));
                let (_, advscore) = self.find_best_move(Rc::clone(&g), p.opponent(), moves_ahead-1, false);
                match advscore {
                    Some(Score::Won(in_n)) => { lost_options.push((Rc::clone(&undecided), in_n+1)); },
                    Some(Score::Remis(in_n)) => { remis_option = Some((Rc::clone(&undecided), in_n+1)); },
                    Some(Score::Lost(in_n)) => { 
                        g.borrow_mut().withdraw_move(p, Rc::clone(&undecided));
                        return (Some(undecided), Some(Score::Won(in_n+1)));
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

        let mut undecided_option: Option<Rc<dyn Move<T>>> = None;
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
            if let Some((remis, in_n)) = remis_option {
                if undecided_pv >= 0.5 { return (Some(undecided), Some(Score::Undecided(undecided_pv))); }
                else { return (Some(remis), Some(Score::Remis(in_n+1))); }
            } else {
                return (Some(undecided), Some(Score::Undecided(undecided_pv)));
            }
        }

        if let Some((remis, in_n)) = remis_option { return (Some(remis), Some(Score::Remis(in_n))); }
        let mut latest_possible = None;
        let mut latest = 0;
        for (lost, in_n) in lost_options {
            if in_n > latest { latest_possible = Some((lost, in_n)); latest = in_n; }
            else {
                match latest_possible {
                     None => latest_possible = Some((lost, in_n)), _ => ()
                }
            }
        }
        if let Some((lost, in_n)) = latest_possible { return (Some(lost), Some(Score::Lost(in_n))); }
        (None, None)
    }
}


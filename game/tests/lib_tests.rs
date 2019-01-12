extern crate game;
use game::*;

use std::rc::Rc;
use std::cell::RefCell;

const TOLERANCE:f32 = 0.0001;

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
                Score::Won(in_n) => {
                    assert!(i>2,i);
                    assert!(in_n==0, in_n); 
                },
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
        Ok(x) => if let Score::Won(0) = x { () } else { assert!(false)},
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
    //println!("{:?}", &cf.field);
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
        assert!(Score::Won(0) == score);
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
                            15 as f32 * strategy.mscore_koeff * strategy.nscore_koeff
                        + 15 as f32 * strategy.oscore_koeff * strategy.nscore_koeff);
        match score {
                Score::Undecided(ev) => assert!(
                    ev == 15 as f32 * strategy.mscore_koeff * strategy.nscore_koeff
                        + 15 as f32 * strategy.oscore_koeff * strategy.nscore_koeff),
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

    let expected = 10 as f32 * s.mscore_koeff * s.nscore_koeff
                    + 0 as f32 * s.mscore_koeff
                    + 10 as f32 * s.oscore_koeff * s.nscore_koeff
                    + 1 as f32 * s.oscore_koeff;
    if let Ok(eval) = s.evaluate_move(Rc::new(RefCell::new(g)), &white, Rc::new(mv)) {
        println!("expected score {} vs calculated {}", expected, eval);
        assert!(eval == expected)
    } else { assert!(false) }

    // test vertical evaluation
    g = ConnectFour::new();
    let _ = g.drop_stone(&black, Column::One);
    let _ = g.drop_stone(&white, Column::One);
    let _ = g.drop_stone(&white, Column::One);
    
    // the example below points to a questionable trait of the evaluation
    // cells are adding up to the score even when they are 'redundant', 
    // i.e. when they belong to a 5-connection in each case they could
    // belong to a 4-connection
    let expected = 6 as f32 * s.mscore_koeff * s.nscore_koeff
                    + 0 as f32 * s.mscore_koeff
                    + 8 as f32 * s.oscore_koeff * s.nscore_koeff
                    + 2 as f32 * s.oscore_koeff;
    if let Ok(eval) = s.evaluate_move(Rc::new(RefCell::new(g)),
                                        &black,
                                        Rc::new(ConnectFourMove { data: Column::One, })) {
        println!("expected score {} vs calculated {}", expected, eval);
        assert!((eval-expected).abs() < TOLERANCE);
    } else { assert!(false) }

}
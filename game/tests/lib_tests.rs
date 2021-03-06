extern crate game;
use game::connectfour::*;
use game::generic::*;
use game::bruteforce::*;

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
    let pm:Vec<Rc<dyn Move<Column>>> = cf.possible_moves(&p);
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
            nscore_koeff: 0.5,
            my_tabu_koeff: 0.0,
            opp_tabu_koeff: 0.0,
            tabu_defense_koeff: 0.0,
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

    // spot traps
    game = ConnectFour::new();
    game.drop_stone(&white, Column::Four).unwrap();
    game.drop_stone(&black, Column::Four).unwrap();
    game.drop_stone(&white, Column::Five).unwrap();
    // below 3 moves ahead premeditation the danger is not recognized!
    if let (Some(mv), Some(score)) = strategy.find_best_move(
            Rc::new(RefCell::new(game)), &black, 3, true) {
        println!("trap {:?} {:?}", mv.data(), score);
        assert!(Score::Undecided(7.8) == score);
        // one may think Three is best - but: computers says no. Three scores 7.5.
        assert!(*mv.data() == Column::Six);
    } else { assert!(false); }

    // spot opportunities
    game = ConnectFour::new();
    game.drop_stone(&white, Column::Four).unwrap();
    game.drop_stone(&black, Column::Four).unwrap();
    game.drop_stone(&white, Column::Five).unwrap();
    game.drop_stone(&black, Column::Five).unwrap();
    if let (Some(mv), Some(score)) = strategy.find_best_move(
            Rc::new(RefCell::new(game)), &white, 4, true) {
        println!("opportunity {:?} {:?}", mv.data(), score);
        assert!(Score::Won(2) == score);
        assert!(*mv.data() == Column::Three);
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
            my_tabu_koeff: 0.0,
            opp_tabu_koeff: 0.0,
            tabu_defense_koeff: 0.0,
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

#[test]
fn test_latest_possible_loss() {
    let x = "------
xxx
ox
xx
xxo
";
    let game = replicate_game(x);
    assert_best_move(None, game, &Player::White, Column::One, Score::Lost(3));

    let x = "------
xxo
xx
ox
xxx
";
    let game = replicate_game(x);
    assert_best_move(None, game, &Player::White, Column::Four, Score::Lost(3));
}

#[test]
fn test_evaluate_tabu_move() {
    let strategy = ConnectFourStrategy {
        mscore_koeff: 0.0,
        oscore_koeff: 0.0,
        nscore_koeff: 0.0,
        my_tabu_koeff: -1.0,
        opp_tabu_koeff: 8.0,
        tabu_defense_koeff: 0.5,
    };
    let game = replicate_game("------
ox
ox
ox
xo
");
    let g = Rc::new(RefCell::new(game));
    let h = g.clone();
    match strategy.evaluate_move(g, &Player::Black, Rc::new(ConnectFourMove { data: Column::Two })) {
        Ok(e) => { println!("{}", e); assert!(e==8.0) },
        _ => assert!(false),
    }
    match strategy.evaluate_move(h.clone(), &Player::White, Rc::new(ConnectFourMove { data: Column::Two })) {
        Ok(e) => { println!("{}", e); assert!(e==0.5) },
        _ => assert!(false),
    }

}

#[test]
fn evaluate_complex_move() {
    let strategy = ConnectFourStrategy {
        mscore_koeff: 1.0,
        oscore_koeff: 0.8,
        nscore_koeff: 0.5,
        my_tabu_koeff: -10.0,
        opp_tabu_koeff: 10.0,
        tabu_defense_koeff: 0.25,
    };

    let game = replicate_game("------
ox
ooxo
x
oxooox
xxoxxx

oxo
------");
    let g = Rc::new(RefCell::new(game));
    for u in 0..7 {
        println!("{:?} {}",
            Column::from_usize(u),
            strategy.evaluate_move(g.clone(),
                &Player::White,
                Rc::new(ConnectFourMove { data: Column::from_usize(u) }
            )).unwrap_or(0.0)
        );
    }
    match strategy.find_best_move(g.clone(), &Player::White, 10, true) {
        (Some(mv), Some(score)) => {
            println!("{:?} {:?}", mv.data(), score);
        },
        _ => assert!(false),
    }
    match strategy.find_best_move(g.clone(), &Player::White, 9, true) {
        (Some(mv), Some(Score::Undecided(score))) => {
            println!("{:?} {:?}", mv.data(), score);
            assert_eq!(*mv.data(), Column::Two);
            assert_eq!(score, 4.0);
        },
        _ => assert!(false),
    }
}

#[test]
fn evaluate_another_complex_move() {
    let strategy = ConnectFourStrategy {
        mscore_koeff: 1.0,
        oscore_koeff: 0.8,
        nscore_koeff: 0.5,
        my_tabu_koeff: -10.0,
        opp_tabu_koeff: 10.0,
        tabu_defense_koeff: 0.25,
    };

    let game = replicate_game("------



oxooox
xxox

ox
------");
    // this one is fixed by taking dead cells int account
    complex_evaluation(game, &strategy, &Player::White, Column::One, 6.7);


    let game = replicate_game("------
o
x
oox
oxoxox
x


------");
    // this is probably the losing move of White in the game against L.
    // bending the strategy's koefficients walks us through the test:
    let strategy = ConnectFourStrategy {
        mscore_koeff: 0.8, //vs 1.0
        oscore_koeff: 1.0, //vs 0.8
        nscore_koeff: 0.5,
        my_tabu_koeff: -10.0,
        opp_tabu_koeff: 10.0,
        tabu_defense_koeff: 0.25,
    };
    complex_evaluation(game, &strategy, &Player::White, Column::Three, 9.4);

    // the disaster:
    let _game = replicate_game("------
o
xxox
xoxxxo
oooxox


o
------");
    let game = replicate_game("------

xxox
x
oooxo



------");
    // ... and the preceding move where the game is lost against 'the app'.
    // admittedly it's really hard to foresee the disaster.
    // bending the strategy's koefficients again walks us through the test:
    let strategy = ConnectFourStrategy {
        mscore_koeff: 0.8, //1.0
        oscore_koeff: 1.2, //0.8
        nscore_koeff: 0.5,
        my_tabu_koeff: -10.0,
        opp_tabu_koeff: 5.0, //10.0
        tabu_defense_koeff: 0.25,
    };
    complex_evaluation(game, &strategy, &Player::White, Column::Four, 11.4);
}

fn complex_evaluation(game:ConnectFour, strategy:&ConnectFourStrategy, player:&Player,
                      expected_column:Column, expected_score:f32) {
    let g = Rc::new(RefCell::new(game));
    for u in 0..7 {
        println!("{:?} {}",
            Column::from_usize(u),
            strategy.evaluate_move(g.clone(),
                player,
                Rc::new(ConnectFourMove { data: Column::from_usize(u) }
            )).unwrap_or(0.0)
        );
    }
    match strategy.find_best_move(g.clone(), player, 4, true) {
        (Some(mv), Some(Score::Undecided(score))) => {
            println!("{:?} {:?}", mv.data(), score);
            assert_eq!(*mv.data(), expected_column);
            assert_eq!(score, expected_score);
        },
        _ => assert!(false),
    }
}

#[test]
fn find_complex_winner() {
    let strategy = ConnectFourStrategy::default();
    let game = replicate_game("------

xx
xoxo
oxooxo
xxoo
ox

------");
    let g = Rc::new(RefCell::new(game));
    match strategy.find_best_move(g.clone(), &Player::White, 6, true) {
        (Some(mv), Some(score)) => {
            println!("{:?} {:?}", mv.data(), score);
            // any winning move is fine, column One does it in 6 
            // or even 8 steps if that many are considered ...
            match score {
                Score::Won(n) => assert_eq!(n, 6),
                _ => assert!(false),
            }
       },
        _ => assert!(false),
    }
    match strategy.find_best_move(g.clone(), &Player::White, 3, true) {
        (Some(mv), Some(score)) => {
            println!("{:?} {:?}", mv.data(), score);
            // ... but under increased pressure the way to victory shortens:
            assert_eq!(*mv.data(), Column::Six);
            match score {
                Score::Won(n) => assert_eq!(n, 2),
                _ => assert!(false),
            }
        },
        _ => assert!(false),
    }
}


fn assert_best_move(strategy: Option<ConnectFourStrategy>,
                    game: ConnectFour, player: &Player,
                    col: Column, score: Score) {
    let strategy = match strategy {
        Some(s) => s,
        None => ConnectFourStrategy {
            mscore_koeff: 1.0,
            oscore_koeff: 0.8,
            nscore_koeff: 0.5,
            my_tabu_koeff: 0.0,
            opp_tabu_koeff: 0.0,
            tabu_defense_koeff: 0.0,
        },
    };
    
    if let (Some(mv), Some(calculated)) = strategy.find_best_move(
            Rc::new(RefCell::new(game)), player, 4, true) {
        println!("{:?} {:?}", *mv.data(), calculated);
        assert!(calculated == score);
        assert!(*mv.data() == col);
    } else { assert!(false); }
}

fn replicate_game(plan: &str) -> ConnectFour {
    let mut g = ConnectFour::new();
    for (i, line) in plan.split("\n").enumerate() {
        match i {
            b if (b > 0 && b < 8) => {
                for c in line.chars() {
                    g.drop_stone(
                        match c {
                            'x' => &Player::Black,
                            'o' => &Player::White,
                            what => { println!("{}, {}", what, i); assert!(false); &Player::Black },
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

#[test]
fn test_bruteforce() {

    let nworker = 1;
    let player = Player::Black;

    let strategy = BruteForceStrategy::new(nworker);
    
    let game = ConnectFour::replicate_game("------


oxox

oxox
xo:x

------");

    // with some wisdom
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 4;
    let (h,s) = hash_from_state(g.clone().borrow().state());
    assert!((h,s) == (209874779512449794048, false), "{} is not 209874779512449794048", h);
    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            println!("{:?}", *mv.data());
            assert!(Column::Four == *mv.data());
            if let Score::Won(n) = score { assert!(n == 4); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/toplimit4").unwrap();
            assert!(dump == expected,
                    std::fs::write("tests/data/toplimit4~", dump));
        },
        _ => { assert!(false); },
    };

    // with no wisdom
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 0;

    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            assert!(Column::Four == *mv.data());
            if let Score::Won(n) = score { assert!(n == 4); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/toplimit0").unwrap();
            assert!(dump == expected,
                std::fs::write("tests/data/toplimit0~", dump).unwrap()
            );
        },
        _ => { assert!(false); },
    };
}

#[test]
fn test_bruteforce_2() {

    let nworker = 1;
    let player = Player::Black;

    let strategy = BruteForceStrategy::new(nworker);
    
    let game = ConnectFour::replicate_game("------

o

xo

x

------");

    // with some wisdom
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 4;
    let (h,s) = hash_from_state(g.clone().borrow().state());
    assert!((h,s) == (2305843421530558464, false));
    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            println!("{:?}", *mv.data());
            assert!(Column::Five == *mv.data());
            if let Score::Won(n) = score { assert!(n == 2); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/toplimit4_2").unwrap();
            assert!(dump == expected,
                    std::fs::write("tests/data/toplimit4_2~", dump));
        },
        _ => { assert!(false); },
    };

    // with no wisdom
    // but - alas! - even no wisdom involves a two steps ahead inquiry
    // explaining the sameness of files toplimit0 and toplimit4
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 0;

    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            assert!(Column::Five == *mv.data());
            if let Score::Won(n) = score { assert!(n == 2); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/toplimit0_2").unwrap();
            assert!(dump == expected,
                std::fs::write("tests/data/toplimit0_2~", dump).unwrap()
            );
        },
        _ => { assert!(false); },
    };
}

#[test]
fn test_graying_1() {
    let expected_before_move_six ="------
xoox
ox

xoox

o
ox
------";
    let expected_after_move_six = "------
xoox
ox

xoox

ox
:x
------";
    let game = ConnectFour::replicate_game(expected_before_move_six);
    let mut mg = game.clone();
    let (_score,grayed) = mg.make_shading_move(&Player::Black, Rc::new(ConnectFourMove { data: Column::Six })).unwrap();
    assert!(mg.display().eq(expected_after_move_six), mg.display());
   
    let (hash, swapped) = hash_from_state(mg.state());
    let (expected_hash, eswapped) = hash_from_state(ConnectFour::replicate_game(expected_after_move_six).state());
    assert!(hash == expected_hash);
    assert!(eswapped && swapped);
    assert!(hash == 708365348734296165224459, "{} is not 708365348734296165224459", hash);

    // undo
    mg.withdraw_move_unshading(&Player::Black, Rc::new(ConnectFourMove { data: Column::Six }), grayed);
    assert!(mg.display().eq(expected_before_move_six), mg.display());
   
    let (hash, swapped) = hash_from_state(mg.state());
    let (expected_hash, eswapped) = hash_from_state(ConnectFour::replicate_game(expected_before_move_six).state());
    assert!(hash == expected_hash);
    assert!(eswapped && swapped);
    assert!(hash == 708365348734296165191689, "{} is not 708365348734296165191689", hash);

    // at last check shading in action
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 0;
    let nworker = 1;
   
    let player = Player::Black;
    let strategy = BruteForceStrategy::new(nworker);
 
    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            assert!(Column::Six == *mv.data());
            if let Score::Won(n) = score { assert!(n == 4); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/shading").unwrap();
            assert!(dump == expected,
                std::fs::write("tests/data/shading~", dump).unwrap()
            );
        },
        _ => { assert!(false); },
    };
}

#[test]
fn test_graying_2() {
    let expected_before_move_two ="------
oxox
x

xo

o

------";
    let expected_after_move_two = "------
:xox
xx

xo

o

------";
    let game = ConnectFour::replicate_game(expected_before_move_two);
    let mut mg = game.clone();
    let (_score,grayed) = mg.make_shading_move(&Player::Black, Rc::new(ConnectFourMove { data: Column::Two })).unwrap();
    assert!(mg.display().eq(expected_after_move_two), mg.display());

    let hash = hash_from_state(mg.state());
    let expected_hash = hash_from_state(ConnectFour::replicate_game(expected_after_move_two).state());
    assert!(hash == expected_hash);

    // undo
    mg.withdraw_move_unshading(&Player::Black, Rc::new(ConnectFourMove { data: Column::Two }), grayed);
    assert!(mg.display().eq(expected_before_move_two), mg.display());

    let hash = hash_from_state(mg.state());
    let expected_hash = hash_from_state(ConnectFour::replicate_game(expected_before_move_two).state());
    assert!(hash == expected_hash);
}

#[test]
fn test_basically_over() {
    let nworker = 1;
    let player = Player::Black;

    let strategy = BruteForceStrategy::new(nworker);
    
    let thirty_stones = "------
:::xo
:::ox
:::xo
:::ox
:::xo
:::ox
:::xo
------";

    unsafe {
        BASICALLY_OVER = 36;
    }

    let game = ConnectFour::replicate_game(thirty_stones);
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 0;

    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            println!("{:?}", mv.data());
            assert!(Column::One == *mv.data());
            if let Score::Remis(n) = score { assert!(n == 6); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/thirtyfivestones").unwrap();
            assert!(dump == expected,
                std::fs::write("tests/data/thirtyfivestones~", dump).unwrap()
            );
        },
        _ => { assert!(false); },
    };

    unsafe {
        BASICALLY_OVER = 30;
    }

    let game = ConnectFour::replicate_game(thirty_stones);
    let g = Rc::new(RefCell::new(game.clone()));
    let toplimit = 0;

    match strategy.find_best_move(g.clone(), &player, toplimit, true) {
        (Some(mv), Some(score)) => {
            assert!(Column::Seven == *mv.data());
            if let Score::Remis(n) = score { assert!(n == 6); }
            else { assert!(false); }

            let dump = std::fs::read_to_string(STRDMP).unwrap();
            let expected = std::fs::read_to_string("tests/data/thirtystones").unwrap();
            assert!(dump == expected,
                std::fs::write("tests/data/thirtystones~", dump).unwrap()
            );
        },
        _ => { assert!(false); },
    };
}

extern crate game;
use game::bruteforce::BruteForceStrategy;
use game::connectfour::*;
use game::generic::*;


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;

fn time_pondering(game:&ConnectFour, nworker:usize, toplimit:i32, player:&Player) -> u64 {
    let strategy = BruteForceStrategy::new(nworker);
    let g = Rc::new(RefCell::new(game.clone()));

    let then = Instant::now();

    match strategy.find_best_move(g.clone(), player, toplimit, true) {
        (Some(mv), Some(score)) => {
            println!("{:?} {:?}", mv.data(), score);
        },
        _ => (),
    }

    let now = Instant::now();
    let tp = now.duration_since(then).as_secs();
    tp

}

fn main() {
    let nworker = 3;
    let toplimit = 6;
    let player = Player::Black;
    let games = [/* ConnectFour::new(), replicate_game("------







------"), */ replicate_game("------

x

xo

o

------"), ];
    let _timep = games.iter()
    .map(|game| {
        time_pondering(game, nworker, toplimit, &player)
    })
    .map(|tp| {
        println!("ran with {} workers, it took {} seconds", nworker, tp);
        tp
    })
    .collect::<Vec<_>>();
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

extern crate game;
use game::bruteforce::BruteForceStrategy;
use game::connectfour::{ConnectFour, Column};
use game::generic::{Player,Strategy};


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;

fn time_pondering(game:&ConnectFour, player:&Player, lookahead:i32, nworker:i32, toplimit:i32) -> u64 {
    let strategy = BruteForceStrategy::new(nworker);
    let g = Rc::new(RefCell::new(game.clone()));

    let then = Instant::now();

    strategy.pave_ground(g.clone(), player, toplimit);

    match strategy.find_best_move(g.clone(), player, lookahead, true) {
        (Some(mv), Some(score)) => {
            println!("{:?} {:?}", mv.data(), score);
        },
        _ => (),
    }

    let now = Instant::now();
    let tp = now.duration_since(then).as_secs();
    tp
}

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

fn main() {
    
    let nworker = 3;
    let lookahead = 4;
    let toplimit = 8;
    let player = Player::White;
    let games = [ConnectFour::new(), replicate_game("------







------"), replicate_game("------

x

xo



------"), ];
    let _timep = games.iter()
    .map(|game| {
        time_pondering(game, &player, lookahead, nworker, toplimit)
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

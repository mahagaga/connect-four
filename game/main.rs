pub mod game;

use game::*;


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;

fn time_pondering(game:ConnectFour, bot:i32, top:i32) {
    let strategy = ConnectFourStrategy::default();
    let g = Rc::new(RefCell::new(game));

    for n in bot..=top {
        let then = Instant::now();

        match strategy.find_best_move(g.clone(), &Player::White, n, true) {
            (Some(mv), Some(score)) => {
                println!("{:?} {:?}", mv.data(), score);
            },
            _ => (),
        }

        let now = Instant::now();
        let tp = now.duration_since(then).as_secs() as u128 * 1000000 + now.duration_since(then).subsec_micros() as u128;
        println!("{} {}", n, tp);

    }
}

fn main() {
    time_pondering(ConnectFour::new(), 7, 8);
    time_pondering(replicate_game("------

x

xo



------"), 7, 8);
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

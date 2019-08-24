extern crate game;
use game::bruteforce::BruteForceStrategy;
use game::connectfour::*;
use game::generic::*;


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;
use std::env;

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

fn default_int(a:Option<&String>, default:usize) -> usize {
    match a {
        Some(n) => match n.parse::<usize>() {
            Ok(u) => u,
            Err(_) => panic!("{} is not a number.", n),
        },
        None => default,
    }
}

fn read_game_from_file(a:Option<&String>) -> ConnectFour {
    let default = "------

x

xo

o

------";
    let plan = match a {
        Some(path) => match std::fs::read_to_string(path) {
            Err(_) => panic!("cannot read file {}", path),
            Ok(string) => string,
        },
        None =>  {
            println!("play default game");
            String::from(default)
        },
    };
    replicate_game(&plan[..])
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let nworker = default_int(args.get(3), 3);
    let toplimit = default_int(args.get(4), 4) as i32;
    let player = match &args.get(2) {
        Some(p) =>  {
            match &p[..] {
                "black" => Player::Black,
                "white" => Player::White,
                _ => panic!("{} neither black nor white. you are black then.", p),
            }
        },
        None => Player::Black,
    };
    
    let game = read_game_from_file(args.get(1));
    let games = [game,];
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

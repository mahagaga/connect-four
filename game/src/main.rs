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

fn game_from_hash(hash:i128) -> ConnectFour {
    let mut game = ConnectFour::new();
    let mut h = hash;
    let base:i128 = 4;
    for ci in 0..ConnectFour::width() {
        let col = Column::from_usize(ci);
        let mut cr = h % base.pow(ConnectFour::height() as u32);
        for _ri in 0..ConnectFour::height() {
            let stone = cr % base;
            match stone {
                1 => { game.drop_stone(&Player::White, col.clone()).unwrap(); },
                2 => { game.drop_stone(&Player::Black, col.clone()).unwrap(); },
                3 => { game.drop_stone(&Player::Gray, col.clone()).unwrap(); },
                0 => break,
                _ => (),
            }
            cr = cr / base;
        }
        h = h / base.pow(ConnectFour::height() as u32);
    }
    game
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
            Err(_) =>  {
                return game_from_hash(default_int(a, 0) as i128);
            },
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
    let toplimit = default_int(args.get(4), 0) as i32;
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
            c if (c==0 || c ==8) => assert_eq!(line, "------"),
            _ => (),
        }
    }
    g
}

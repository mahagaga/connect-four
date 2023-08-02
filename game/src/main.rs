extern crate game;
use game::bruteforce::{BruteForceStrategy,LIMIT,BASICALLY_OVER};
use game::connectfour::*;
use game::generic::*;


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;
use std::env;

fn time_pondering(game:&ConnectFour, nworker:usize, moves_ahead:i32, player:&Player) -> u64 {
    let g = Rc::new(RefCell::new(game.clone()));

    let then = Instant::now();

    let result = match nworker {
        0 => ConnectFourStrategy::default().find_best_move(g.clone(), player, moves_ahead, true),
        n => BruteForceStrategy::new(n).find_best_move(g.clone(), player, moves_ahead, true),    
    };
    match result {
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
                let h = match path.parse::<i128>() {
                    Ok(u) => u,
                    Err(_) => panic!("{} neither file nor hash.", path),
                };
                return game_from_hash(h);
            },
            Ok(string) => string,
        },
        None =>  {
            println!("play default game");
            String::from(default)
        },
    };
    ConnectFour::replicate_game(&plan[..])
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let nworker = default_int(args.get(3), 3);
    let moves_ahead = default_int(args.get(4), 4) as i32;
    let game = read_game_from_file(args.get(1));
    unsafe {
        BASICALLY_OVER = default_int(args.get(5), 30) as usize;
        LIMIT = default_int(args.get(6), 0) as u128;
    }
    let player = match &args.get(2) {
        Some(p) =>  {
            match &p[..] {
                "black" => Player::Black,
                "white" => Player::White,
                "show" => { println!("{}",game.display()); return; }
                _ => panic!("{} neither black nor white. you are black then.", p),
            }
        },
        None => Player::Black,
    };
    
    let games = [game,];
    let _timep = games.iter()
    .map(|game| {
        time_pondering(game, nworker, moves_ahead, &player)
    })
    .map(|tp| {
        println!("ran with {} workers, it took {} seconds", nworker, tp);
        tp
    })
    .collect::<Vec<_>>();
}



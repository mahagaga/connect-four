extern crate game;
use game::bruteforce::BruteForceStrategy;
use game::connectfour::ConnectFour;
use game::generic::{Player,Strategy};


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;

fn time_pondering(game:&ConnectFour, player:&Player, lookahead:u32, nworker:usize, toplimit:u32) -> u64 {
    let strategy = BruteForceStrategy::new(nworker);
    let g = Rc::new(RefCell::new(game.clone()));

    let then = Instant::now();

    strategy.pave_ground(g.clone(), player, toplimit);
    // TODO: change lookahead to u32!
    match strategy.find_best_move(g.clone(), player, lookahead as i32, true) {
        (Some(mv), Some(score)) => {
            println!("{:?} {:?}", mv.data(), score);
        },
        _ => (),
    }
    let store = strategy.collect_store();
    println!("store size {}", store.scores.keys().len());
    
    let now = Instant::now();
    let tp = now.duration_since(then).as_secs();
    tp
}

fn main() {
    
    let nworker = 3;
    let lookahead = 4;
    let toplimit = 16;
    let player = Player::White;
    let games = [ConnectFour::new(), ConnectFour::replicate(String::from("------







------")), ConnectFour::replicate(String::from("------

x

xo



------")), ];
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

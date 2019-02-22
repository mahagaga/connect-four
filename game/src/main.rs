extern crate game;
use game::bruteforce::BruteForceStrategy;
use game::connectfour::ConnectFour;
use game::generic::Player;


use std::time::{Instant};
use std::rc::Rc;
use std::cell::RefCell;

fn time_pondering(game:&ConnectFour, player:&Player, lookahead:u32, nworker:usize, toplimit:u32) -> u64 {
    let strategy = BruteForceStrategy::new(nworker, lookahead);
    let g = Rc::new(RefCell::new(game.clone()));

    let then = Instant::now();

    strategy.pave_ground(g.clone(), player, toplimit);
    
    let store = strategy.collect_store();
    println!("store size {}", store.scores.keys().len());
    
    let now = Instant::now();
    let tp = now.duration_since(then).as_secs();
    tp
}

use std::env;
fn main() {
    let toplimit = match env::args().nth(1) {
        Some(n) => n.parse::<u32>().unwrap(),
        None => 16,
    };
    let lookahead = match env::args().nth(2) {
        Some(n) => n.parse::<u32>().unwrap(),
        None => 4,
    };;
    let nworker = match env::args().nth(3) {
        Some(n) => n.parse::<usize>().unwrap(),
        None => 3,
    };
    let player = Player::White;
    let games = [ConnectFour::replicate(String::from("------



ox



------")),];
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

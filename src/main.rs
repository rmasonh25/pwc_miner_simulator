use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::thread;
use std::time::Instant;

const NUM_MINERS: u32 = 4;
const SHARE_DIFFICULTY: usize = 7;
const POOL_DIFFICULTY: usize = 12;

fn double_sha256(input: &str) -> String {
    let first = Sha256::digest(input.as_bytes());
    let second = Sha256::digest(&first);
    hex::encode(second)
}

fn meets_difficulty(hash: &str, difficulty: usize) -> bool {
    hash.chars().take(difficulty).all(|c| c == '0')
}

fn simulate_miner(
    miner_id: u32,
    block_header: &str,
    start_nonce: u32,
    range: u32,
    found: Arc<AtomicBool>,
    result: Arc<AtomicU32>,
    shares: Arc<Mutex<HashMap<u32, u32>>>
) {
    for nonce in start_nonce..start_nonce + range {
        if found.load(Ordering::Relaxed) {
            break;
        }
        let data = format!("{}{}", block_header, nonce);
        let hash = double_sha256(&data);

        // Submit a share
        if meets_difficulty(&hash, SHARE_DIFFICULTY) {
            let mut s = shares.lock().unwrap();
            *s.entry(miner_id).or_insert(0) += 1;
        }

        // Check if it solves the pool difficulty
        if meets_difficulty(&hash, POOL_DIFFICULTY) {
            found.store(true, Ordering::Relaxed);
            result.store(nonce, Ordering::Relaxed);
            println!("[Miner {}] 🔥 Found block! Nonce: {} Hash: {}", miner_id, nonce, hash);
            break;
        }
    }
}

fn run_shared_mining_simulation() {
    let block_header = "SIMULATED_BLOCK";
    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(AtomicU32::new(0));
    let shares = Arc::new(Mutex::new(HashMap::new()));

    let nonce_range: u32 = 100_000_000;

    println!("🔁 Starting share-based mining simulation");
    println!("⛏️  {} miners, Share Difficulty: {}, Pool Difficulty: {}\n", NUM_MINERS, SHARE_DIFFICULTY, POOL_DIFFICULTY);

    let start_time = Instant::now();
    let mut handles = vec![];

    for i in 0..NUM_MINERS {
        let start_nonce = i * nonce_range;
        let header = block_header.to_string();
        let f = Arc::clone(&found);
        let r = Arc::clone(&result);
        let s = Arc::clone(&shares);

        handles.push(thread::spawn(move || {
            simulate_miner(i, &header, start_nonce, nonce_range, f, r, s);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start_time.elapsed();
    println!("\n🕒 Block round ended in {:.2?}", elapsed);

    // Display share stats
    let s = shares.lock().unwrap();
    println!("\n📊 Miner Share Contributions:");
    for (miner, count) in s.iter() {
        println!(" - Miner {}: {} shares", miner, count);
    }
}

fn main() {
    run_shared_mining_simulation();
}


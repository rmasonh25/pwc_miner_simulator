// Rust-based MVP mining simulator
// Simulates a pool with 4 local threads (miners) mining a simplified block
// Difficulty is tunable (number of leading zeros)

use sha2::{Digest, Sha256};
use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::thread;
use std::time::Instant;

// Customize this to simulate different difficulties (e.g., 3 = "000")
const DIFFICULTY: usize = 4;
const RANGE_SIZE: u32 = 25_000_000; // Total nonce range (divided across 4 miners)
const NUM_MINERS: u32 = 4;

fn double_sha256(input: &str) -> String {
    let first = Sha256::digest(input.as_bytes());
    let second = Sha256::digest(&first);
    hex::encode(second)
}

fn meets_difficulty(hash: &str, difficulty: usize) -> bool {
    hash.chars().take(difficulty).all(|c| c == '0')
}

fn simulate_miner(miner_id: u32, block_header: &str, start_nonce: u32, range: u32, found: Arc<AtomicBool>, result: Arc<AtomicU32>) {
    for nonce in start_nonce..start_nonce + range {
        if found.load(Ordering::Relaxed) {
            break;
        }
        let data = format!("{}{}", block_header, nonce);
        let hash = double_sha256(&data);
        if meets_difficulty(&hash, DIFFICULTY) {
            found.store(true, Ordering::Relaxed);
            result.store(nonce, Ordering::Relaxed);
            println!("[Miner {}] Found block! Nonce: {} Hash: {}", miner_id, nonce, hash);
            break;
        }
    }
}

fn main() {
    let block_header = "MVP_SIMULATED_BLOCK_HEADER";
    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];

    let nonce_range = RANGE_SIZE / NUM_MINERS;
    let start_time = Instant::now();

    println!("Starting mining simulation with {} miners and difficulty {}\n", NUM_MINERS, DIFFICULTY);

    for i in 0..NUM_MINERS {
        let start_nonce = i * nonce_range;
        let block_header = block_header.to_string();
        let found = Arc::clone(&found);
        let result = Arc::clone(&result);

        handles.push(thread::spawn(move || {
            simulate_miner(i, &block_header, start_nonce, nonce_range, found, result);
        }));
    }

    for h in handles { h.join().unwrap(); }

    let elapsed = start_time.elapsed();
    if found.load(Ordering::Relaxed) {
        let nonce = result.load(Ordering::Relaxed);
        println!("\n✅ Block found! Nonce: {}", nonce);
    } else {
        println!("\n❌ Block not found in range");
    }
    println!("⏱️ Elapsed time: {:.2?}", elapsed);
}


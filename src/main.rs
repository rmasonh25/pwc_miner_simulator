// Rust-based MVP mining simulator with auto-adjusting difficulty
// Adjusts difficulty until one iteration takes close to 10 minutes

use sha2::{Digest, Sha256};
use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

const NUM_MINERS: u32 = 4;
const RANGE_SIZE: u32 = 100_000_000; // Nonce range per run (can be increased)
const TARGET_TIME: u64 = 600; // Target time in seconds (10 minutes)

fn double_sha256(input: &str) -> String {
    let first = Sha256::digest(input.as_bytes());
    let second = Sha256::digest(&first);
    hex::encode(second)
}

fn meets_difficulty(hash: &str, difficulty: usize) -> bool {
    hash.chars().take(difficulty).all(|c| c == '0')
}

fn simulate_miner(miner_id: u32, block_header: &str, start_nonce: u32, range: u32, difficulty: usize, found: Arc<AtomicBool>, result: Arc<AtomicU32>) {
    for nonce in start_nonce..start_nonce + range {
        if found.load(Ordering::Relaxed) {
            break;
        }
        let data = format!("{}{}", block_header, nonce);
        let hash = double_sha256(&data);
        if meets_difficulty(&hash, difficulty) {
            found.store(true, Ordering::Relaxed);
            result.store(nonce, Ordering::Relaxed);
            println!("[Miner {}] Found block! Nonce: {} Hash: {}", miner_id, nonce, hash);
            break;
        }
    }
}

fn run_simulation(difficulty: usize) -> Duration {
    let block_header = "MVP_SIMULATED_BLOCK_HEADER";
    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];
    let nonce_range = RANGE_SIZE / NUM_MINERS;

    println!("\n🔁 Starting mining simulation with difficulty {}", difficulty);
    let start_time = Instant::now();

    for i in 0..NUM_MINERS {
        let start_nonce = i * nonce_range;
        let block_header = block_header.to_string();
        let found = Arc::clone(&found);
        let result = Arc::clone(&result);

        handles.push(thread::spawn(move || {
            simulate_miner(i, &block_header, start_nonce, nonce_range, difficulty, found, result);
        }));
    }

    for h in handles { h.join().unwrap(); }

    let elapsed = start_time.elapsed();
    if found.load(Ordering::Relaxed) {
        let nonce = result.load(Ordering::Relaxed);
        println!("✅ Block found at nonce {} in {:.2?} (difficulty {})", nonce, elapsed, difficulty);
    } else {
        println!("❌ No block found at difficulty {} in {:.2?}", difficulty, elapsed);
    }

    elapsed
}

fn main() {
    let mut difficulty = 3; // Starting point

    loop {
        let duration = run_simulation(difficulty);
        let seconds = duration.as_secs();

        if seconds < TARGET_TIME / 2 {
            difficulty += 1;
        } else if seconds > TARGET_TIME * 2 {
            difficulty = difficulty.saturating_sub(2);
        } else if seconds > TARGET_TIME {
            difficulty = difficulty.saturating_sub(1);
        } else {
            println!("🎯 Reached target duration at difficulty {}", difficulty);
            break;
        }
    }

    println!("⛏️ Final adjusted difficulty: {}", difficulty);
}


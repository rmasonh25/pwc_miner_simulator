// Rust-based simulator with:
// 1. Dynamic range scaling
// 2. Hashrate estimation
// 3. CSV performance logging

use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}, Mutex};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

const NUM_MINERS: u32 = 4;
const BASE_RANGE: u32 = 25_000_000; // Base range per difficulty unit
const TARGET_TIME: u64 = 600; // 10 minutes
const CSV_FILE: &str = "results.csv";
const NONCE_STEP: u32 = 1_000_000; // Custom nonce step value for testing

fn double_sha256(input: &str) -> String {
    let first = Sha256::digest(input.as_bytes());
    let second = Sha256::digest(&first);
    hex::encode(second)
}

fn meets_difficulty(hash: &str, difficulty: usize) -> bool {
    hash.chars().take(difficulty).all(|c| c == '0')
}

fn simulate_miner(miner_id: u32, block_header: &str, start_nonce: u32, range: u32, difficulty: usize, found: Arc<AtomicBool>, result: Arc<AtomicU32>, hash_count: Arc<AtomicU64>) {
    let mut nonce = start_nonce;
    let end_nonce = start_nonce + range;
    while nonce < end_nonce {
        if found.load(Ordering::Relaxed) {
            break;
        }
        let data = format!("{}{}", block_header, nonce);
        let hash = double_sha256(&data);
        hash_count.fetch_add(1, Ordering::Relaxed);
        if meets_difficulty(&hash, difficulty) {
            found.store(true, Ordering::Relaxed);
            result.store(nonce, Ordering::Relaxed);
            println!("[Miner {}] Found block! Nonce: {} Hash: {}", miner_id, nonce, hash);
            break;
        }
        nonce += NONCE_STEP;
    }
}

fn run_simulation(difficulty: usize) -> (Duration, u64, bool) {
    let block_header = "MVP_SIMULATED_BLOCK_HEADER";
    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(AtomicU32::new(0));
    let hash_count = Arc::new(AtomicU64::new(0));

    let range_size = BASE_RANGE * difficulty as u32;
    let nonce_range = range_size / NUM_MINERS;
    let mut handles = vec![];

    println!("\n🔁 Starting simulation | Difficulty: {} | Range: {} per thread", difficulty, nonce_range);
    let start_time = Instant::now();

    for i in 0..NUM_MINERS {
        let start_nonce = i * nonce_range;
        let block_header = block_header.to_string();
        let found = Arc::clone(&found);
        let result = Arc::clone(&result);
        let hash_count = Arc::clone(&hash_count);

        handles.push(thread::spawn(move || {
            simulate_miner(i, &block_header, start_nonce, nonce_range, difficulty, found, result, hash_count);
        }));
    }

    for h in handles { h.join().unwrap(); }
    let elapsed = start_time.elapsed();
    let total_hashes = hash_count.load(Ordering::Relaxed);
    let success = found.load(Ordering::Relaxed);

    if success {
        println!("✅ Block found in {:.2?}", elapsed);
    } else {
        println!("❌ Block not found in {:.2?}", elapsed);
    }

    (elapsed, total_hashes, success)
}

fn append_to_csv(difficulty: usize, elapsed: Duration, hashes: u64, found: bool) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(CSV_FILE)
        .expect("Failed to open results.csv");

    let seconds = elapsed.as_secs_f64();
    let hash_rate = hashes as f64 / seconds / 1_000_000.0; // MH/s
    let status = if found { "FOUND" } else { "NOT FOUND" };

    writeln!(file, "{},{:.2},{},{:.2},{}", difficulty, seconds, hashes, hash_rate, status)
        .expect("Failed to write to CSV");
}

fn main() {
    let mut difficulty = 3;

    println!("difficulty,time(s),hashes,hashrate(MH/s),status");

    loop {
        let (elapsed, hashes, found) = run_simulation(difficulty);
        append_to_csv(difficulty, elapsed, hashes, found);

        let seconds = elapsed.as_secs();

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


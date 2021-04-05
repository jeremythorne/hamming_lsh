use rand::Rng;
use std::time::SystemTime;
use hamming_lsh::{HammingCode, HammingLSH, hamming_peturb, hamming_distance};

fn test_lsh(k:u32, l:u32, v:&Vec<HammingCode>, f:u32) {

    let mut lsh = HammingLSH::new(k, l);
    
    let now = SystemTime::now();
    for (i, j) in v.iter().enumerate() {
        lsh.insert(*j, i);
    }
    let insertion_time = now.elapsed().unwrap().as_millis();

    let v_p: Vec<(usize, u128)> = v.iter()
        .map(|a| hamming_peturb(*a, f))
        .enumerate()
        .collect();
    
    let now = SystemTime::now();
    let mut sum = 0;
    let mut distance = 0.0;
    for (i, a) in v_p.iter() {
        if let Some(n) = lsh.get(*a) {
            if *n.1 == *i {
                sum += 1;
                distance += hamming_distance(n.0, *a) as f64;
            }
        }
    }
    let lookup_time = now.elapsed().unwrap().as_millis();
    let match_rate = (sum as f64 * 100.0) / (v.len() as f64);
    let average_distance = distance / ( sum as f64);
    println!("k {}, l {}, matched {}, avg dist {}, insertion {}ms, lookup {}ms",
             k, l, match_rate, average_distance, insertion_time, lookup_time);
}

fn main() {
    println!("Hello, world!");
    const N:usize = 10000;
    const F:u32 = 4;
    let v: Vec<HammingCode> = rand::thread_rng()
        .sample_iter(&rand::distributions::Standard)
        .take(N)
        .collect();

    println!("inserting {} values into different sized LHS, peturbing {} bits and then performing lookup",
             N, F);

    for k in 1..16 {
        for l in 1..8 {
            test_lsh(k, l, &v, F);
        }
    }
}

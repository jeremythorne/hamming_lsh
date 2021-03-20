use rand::Rng;
use rand::seq::SliceRandom;
use std::time::SystemTime;

type HammingCode = u128;

fn hamming_distance(a:HammingCode, b: HammingCode) -> u32 {
    return (a ^ b).count_ones() as u32
}

struct HammingTable {
    hyperplanes: Vec<HammingCode>,
    buckets: Vec<Vec<HammingCode>>
}

impl HammingTable {
    fn new(k: u32) -> HammingTable {
        let mut b: Vec<u32> = (0..128).collect();
        b.shuffle(&mut rand::thread_rng());

        let hyperplanes: Vec<HammingCode> = 
            b[0..k as usize].iter().map(|a| 1 << a).collect();
        let buckets = vec!(Vec::<HammingCode>::new(); 1 << k as usize);

        HammingTable {
            hyperplanes: hyperplanes,
            buckets: buckets
        }
    }

    fn hash(&self, v: HammingCode) -> u32 {
        let mut hash = 0;
        for (i, plane) in self.hyperplanes.iter().enumerate() {
            let h = match v & plane {
                0 => 0,
                _ => 1
            };
            hash |= h << i;
        }
        hash
    }

    fn insert(&mut self, v: HammingCode) {
        let h = self.hash(v);
        self.buckets[h as usize].push(v);
    }

    fn get(&self, v: HammingCode) -> HammingCode {
        let h = self.hash(v);
        let mut min = u32::MAX;
        let mut c:HammingCode = 0;
        for n in self.buckets[h as usize].iter() {
            let d = hamming_distance(*n, v);
            if d < min {
                min = d;
                c = *n;
            }
        }
        c
    }
}

struct HammingLSH {
    tables: Vec<HammingTable>
}

impl HammingLSH {
    fn new(k: u32, l: u32) -> HammingLSH{
        let mut tables = Vec::<HammingTable>::new();
        for _ in 0..l {
            tables.push(HammingTable::new(k));
        }
        HammingLSH {
            tables: tables
        }
    }

    fn insert(&mut self, v: HammingCode) {
        for t in self.tables.iter_mut() {
            t.insert(v);
        }
    }

    fn get(&self, v: HammingCode) -> HammingCode {
        let mut min = u32::MAX;
        let mut c:HammingCode = 0;
        for n in self.tables.iter().map(|t| t.get(v)) {
            let d = hamming_distance(n, v);
            if d < min {
                min = d;
                c = n;
            }
        }
        c
    }
}

fn hamming_peturb(v: HammingCode, bits:u32) -> HammingCode {
    let mut b: Vec<u32> = (0..128).collect();
    b.shuffle(&mut rand::thread_rng());

    let mut a = v;

    for i in b[0..bits as usize].iter() {
        a ^= 1 << i;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_distance() {
        assert_eq!(hamming_distance(0, u128::MAX), 128);
        assert_eq!(hamming_distance(0b101u128, 0b011u128), 2);
    }
}

fn main() {
    println!("Hello, world!");
    const N:usize = 10000;
    const L:u32 = 4;
    const K:u32 = 7; // we make a vec of length 1 << K
    const F:u32 = 4;
    println!("inserting {} 128 bit hamming codes into a Locality Sensitive Hash with {} tables each with {} hyperplanes",
             N, L, K);
    let v: Vec<HammingCode> = rand::thread_rng()
        .sample_iter(&rand::distributions::Standard)
        .take(N)
        .collect();

    let mut lsh = HammingLSH::new(K, L);
    
    let now = SystemTime::now();
    for i in v.iter() {
        lsh.insert(*i)
    }
    println!("insertion took {}ms", now.elapsed().unwrap().as_millis());

    println!("for each of the {} entries, perturbing {} bits and then looking in the LSH for a match",
             N, F);

    let v_p: Vec<(u128, u128)> = v.iter()
        .map(|a| (*a, hamming_peturb(*a, F)))
        .collect();
    
    let now = SystemTime::now();
    let mut sum = 0;
    let mut distance = 0.0;
    for (i, a) in v_p.iter() {
        let n = lsh.get(*a);
        if n == *i {
            sum += 1;
            distance += hamming_distance(n, *a) as f64;
        }
    }
    println!("lookup took {}ms", now.elapsed().unwrap().as_millis());

    println!("matched {} out of {}", sum, N);
    println!("average distance of matches {}", distance / ( sum as f64));
}

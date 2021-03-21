use rand::Rng;
use rand::seq::SliceRandom;
use std::time::SystemTime;

type HammingCode = u128;

fn hamming_distance(a:HammingCode, b: HammingCode) -> u32 {
    return (a ^ b).count_ones() as u32
}

fn hash(planes: &[HammingCode], v: HammingCode) -> u32 {
    let mut hash = 0;
    for (i, plane) in planes.iter().enumerate() {
        let h = match v & plane {
            0 => 0,
            _ => 1
        };
        hash |= h << i;
    }
    hash
}

struct HammingTable<T> {
    hyperplanes: Vec<HammingCode>,
    buckets: Vec<Vec<(HammingCode, T)>>
}

impl<T:Clone> HammingTable<T> {
    fn new(k: u32) -> HammingTable<T> {
        let mut b: Vec<u32> = (0..128).collect();
        b.shuffle(&mut rand::thread_rng());

        let hyperplanes: Vec<HammingCode> = 
            b[0..k as usize].iter().map(|a| 1 << a).collect();
        let buckets = vec!(Vec::<(HammingCode, T)>::new(); 1 << k as usize);

        HammingTable {
            hyperplanes: hyperplanes,
            buckets: buckets
        }
    }

    fn hash(&self, v: HammingCode) -> u32 {
        hash(&self.hyperplanes, v)
    }

    fn insert(&mut self, k: HammingCode, v:T) {
        let h = self.hash(k);
        self.buckets[h as usize].push((k, v));
    }

    fn get(&self, k: HammingCode) -> Option<&(HammingCode, T)> {
        let h = self.hash(k);
        let mut min = u32::MAX;
        let mut best:Option<&(HammingCode, T)> = None;
        for n in self.buckets[h as usize].iter() {
            let d = hamming_distance(n.0, k);
            if d < min {
                min = d;
                best = Some(n);
            }
        }
        best
    }
}

struct HammingLSH<T> {
    tables: Vec<HammingTable<usize>>,
    data: Vec<T>
}

impl<T:Clone> HammingLSH<T> {
    fn new(k: u32, l: u32) -> HammingLSH<T> {
        let mut tables = Vec::<HammingTable<usize>>::new();
        for _ in 0..l {
            tables.push(HammingTable::<usize>::new(k));
        }
        HammingLSH {
            tables: tables,
            data: Vec::<T>::new()
        }
    }

    fn insert(&mut self, k: HammingCode, v:T) {
        let i = self.data.len();
        self.data.push(v);
        for t in self.tables.iter_mut() {
            t.insert(k, i);
        }
    }

    fn get(&self, v: HammingCode) -> Option<(HammingCode, &T)> {
        let mut min = u32::MAX;
        let mut best:Option<(HammingCode, &T)> = None;
        for n in self.tables.iter().map(|t| t.get(v)) {
            if let Some((k, i)) = n {
                let d = hamming_distance(*k, v);
                if d < min {
                    min = d;
                    best = Some((*k, &self.data[*i]));
                }
            }
        }
        best
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

    #[test]
    fn test_hash() {
        assert_eq!(hash(&[0b01u128], 0b1u128), 1);
        assert_eq!(hash(&[0b10u128], 0b1u128), 0);
        assert_eq!(hash(&[0b01u128, 0b10u128], 0b10u128), 0b10);
        assert_eq!(hash(&[1u128 << 127], 1u128 << 127), 1);
    }
}

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

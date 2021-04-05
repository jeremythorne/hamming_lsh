use rand::seq::SliceRandom;

pub type HammingCode = u128;

pub fn hamming_distance(a:HammingCode, b: HammingCode) -> u32 {
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

fn nearest<T: Copy> (candidates:&[Option<(HammingCode, T)>], v: HammingCode) -> Option<(HammingCode, T)> {
    let mut min = u32::MAX;
    let mut best:Option<(HammingCode, T)> = None;
    for n in candidates.iter() {
        if let Some((k, i)) = n {
            let d = hamming_distance(*k, v);
            if d < min {
                min = d;
                best = Some((*k, *i));
            }
        }
    }
    best
}

struct HammingTable<T> {
    hyperplanes: Vec<HammingCode>,
    buckets: Vec<Vec<Option<(HammingCode, T)>>>
}

impl<T:Clone + Copy> HammingTable<T> {
    fn new(k: u32) -> HammingTable<T> {
        let mut b: Vec<u32> = (0..128).collect();
        b.shuffle(&mut rand::thread_rng());

        let hyperplanes: Vec<HammingCode> = 
            b[0..k as usize].iter().map(|a| 1 << a).collect();
        let buckets = vec!(Vec::<Option<(HammingCode, T)>>::new(); 1 << k as usize);

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
        self.buckets[h as usize].push(Some((k, v)));
    }

    fn get(&self, k: HammingCode) -> Option<(HammingCode, T)> {
        let h = self.hash(k);
        nearest(&self.buckets[h as usize][..], k)
    }
}

pub struct HammingLSH<T> {
    tables: Vec<HammingTable<usize>>,
    data: Vec<T>
}

impl<T:Clone> HammingLSH<T> {
    pub fn new(k: u32, l: u32) -> HammingLSH<T> {
        let mut tables = Vec::<HammingTable<usize>>::new();
        for _ in 0..l {
            tables.push(HammingTable::<usize>::new(k));
        }
        HammingLSH {
            tables: tables,
            data: Vec::<T>::new()
        }
    }

    pub fn insert(&mut self, k: HammingCode, v:T) {
        let i = self.data.len();
        self.data.push(v);
        for t in self.tables.iter_mut() {
            t.insert(k, i);
        }
    }

    pub fn get(&self, v: HammingCode) -> Option<(HammingCode, &T)> {
        let c:Vec<Option<(HammingCode, usize)>> = self.tables.iter()
            .map(|t| t.get(v))
            .collect();
        match nearest(&c[..], v) {
            Some((k, i)) => Some((k, &self.data[i])),
            _ => None
        }
    }
}

pub fn hamming_peturb(v: HammingCode, bits:u32) -> HammingCode {
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

    #[test]
    fn test_nearest() {
        assert_eq!(nearest(&[Some((0b110, 0)),
                             Some((0b001, 1))], 0b001),
                                Some((0b001, 1)));
        assert_eq!(nearest::<u32>(&[None], 0b001), None);
    }
}

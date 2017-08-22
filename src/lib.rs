// use std::collections::hash_map::DefaultHasher;
extern crate siphasher;

use std::hash::{Hash, Hasher};

use siphasher::sip::SipHasher;

pub const SMALL_M: usize = 65537;
pub const BIG_M: usize = 655373;

pub struct Maglev<T: Hash + PartialEq + Copy> {
    // VIPs added
    nodes: Vec<T>,

    // size of lookup table
    m: usize,

    lookup: Vec<i64>,
    permutations: Vec<Vec<usize>>,
}

const SIP_OFFSET_KEY: u64 = 0xdeadbabe;
const SIP_SKIP_KEY: u64 = 0xdeadbeef;

impl<T> Maglev<T>
where
    T: Hash + PartialEq + Copy,
{
    pub fn new(m: usize) -> Maglev<T> {
        let mut mh = Maglev {
            nodes: Vec::new(),
            m: m,

            lookup: Vec::new(),
            permutations: Vec::new(),
        };

        mh.generate_population();
        mh.populate();
        mh
    }

    fn generate_population(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        for node in &self.nodes {
            let mut s = SipHasher::new_with_keys(SIP_OFFSET_KEY, 0);
            node.hash(&mut s);
            let offset: usize = s.finish() as usize % self.m;

            let mut s = SipHasher::new_with_keys(SIP_SKIP_KEY, 0);
            node.hash(&mut s);
            let skip: usize = (s.finish() as usize % (self.m - 1)) + 1;

            // TODO: iter + extend
            let mut row: Vec<usize> = Vec::new();
            for j in 0usize..self.m {
                row.push((offset + j * skip) % self.m);
            }

            self.permutations.push(row);
        }
    }

    fn populate(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        let mut next: Vec<usize> = vec![0; self.nodes.len()];
        let mut entry: Vec<i64> = vec![-1; self.m];
        let mut n = 0usize;

        loop {
            for i in 0..self.nodes.len() {
                let mut c = self.permutations[i][next[i]];
                while entry[c] >= 0 {
                    next[i] += 1;
                    c = self.permutations[i][next[i]];
                }

                entry[c] = i as i64;
                next[i] += 1;
                n += 1;
                if n == self.m {
                    self.lookup = entry;
                    return;
                }
            }
        }
    }


    // TODO: return error
    // TODO: clone?
    pub fn add(&mut self, item: T) {
        if self.nodes.contains(&item) {
            return;
        }

        self.nodes.push(item);
        self.generate_population();
        self.populate();
    }

    // TODO: return error
    pub fn remove(&mut self, item: &T) {
        match self.nodes.iter().position(|x| x == item) {
            Some(index) => {
                self.nodes.swap_remove(index);
            }
            None => return,
        };

        self.generate_population();
        self.populate();
    }

    pub fn get(&self, item: &T) -> Option<T> {
        if self.nodes.is_empty() {
            return None;
        }

        let mut s = SipHasher::new_with_keys(SIP_OFFSET_KEY, 0);
        item.hash(&mut s);
        let index = self.lookup[s.finish() as usize % self.lookup.len()];
        Some(self.nodes[index as usize])
    }
}

#[cfg(test)]
mod tests {
    use Maglev;
    use SMALL_M;
    #[test]
    fn fair_distribution() {
        let size: i64 = 10;

        let mut m: Maglev<i64> = Maglev::new(SMALL_M);
        for i in 0..size {
            m.add(i)
        }

        let mut r = vec![0, size];
        for j in 0..10000 {
            match m.get(&j) {
                Some(index) => (),
                None => (),
            }
        }
    }
}

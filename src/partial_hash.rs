use util::{hash as util_hash, bit as ith_bit, highest_set_bit };
use query::Query;
use choice_vec::ChoiceVec;

pub const FULL_MASK: u32 = 0b11111111_11111111_11111111_11111111;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PartialHash {
    /// The multiattribute hash value, if a bit
    /// is unknown it'll be zero, but should be
    /// ignored either way.
    pub hash: u32,
    /// Used to track which bits are being used
    pub mask: u32,
}

#[derive(Debug)]
pub enum HubState {
    HubOff,
    HubSet,
    HubUnknownA,
    HubUnknownB(u32),
}

#[derive(Debug)]
pub struct PageIdIter {
    state: HubState,
    /// The number of yielded page_ids
    current: u32,
    /// The maximum number of bits to consider, equal to d + 1, where d is the relation depth.
    highest_usable_bit: u8,
    /// The number of pages
    num_pages: u32,
    /// Initial hash value, ambiguous bits will be 0
    hash_init: u32,
    /// Used to check which bits are ambiguous
    mask: u32,
    /// The max number of page_ids to yield
    max: u32,
}

impl PartialHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
    }

    pub fn matching_page_ids(&self, num_pages: u32) -> PageIdIter {
        PageIdIter::new(self, num_pages)
    }

    pub fn from_query(query: &Query, choice: &ChoiceVec) -> PartialHash {
        let mut query_hash: u32 = 0;
        let mut query_mask: u32 = FULL_MASK;

        // hash the queries known attributes
        let attr_hashs: Vec<Option<u32>> = {
            let func = |&attr_match: &Option<&str>| attr_match.map(util_hash);
            query.matches.iter().map(func).collect()
        };

        // generate multiattribute hash, from attr_hashs & choice vector,
        // along with the mask signifying which bits are known & unknown
        for (q_bit, &(a_index, a_bit)) in choice.data.iter().enumerate() {
            if let Some(a_hash) = attr_hashs[a_index as usize] {
                // added a'th bit of attribute to hash
                query_hash |= ith_bit(a_bit, a_hash) << q_bit;
            } else {
                // remove the q'th bit from the query mask
                query_mask &= !(1 << q_bit);
            }
        }

        PartialHash { hash: query_hash, mask: query_mask }
    }
}

impl PageIdIter {
    fn new(ma_hash: &PartialHash, num_pages: u32) -> Self {
        let highest_usable_bit = highest_set_bit(num_pages) - 1;

        // used to calculate the max number of iterations
        let mut iterations = 1;

        // iter_mask is a mask to remove any trailing bits
        // from the iterator's mask & init_hash
        let mut iter_mask = 0;

        for i in 0..highest_usable_bit {
            let position = 1 << i;
            iter_mask |= position;
            if ma_hash.mask & position == 0 {
                iterations <<= 1;
            }
        }

        // HUB stands for highest usable bit
        let mask_hub = ith_bit(highest_usable_bit, ma_hash.mask);
        let hash_hub = ith_bit(highest_usable_bit, ma_hash.hash);

        PageIdIter {
            // this is the starting branch of the iterator
            state: match (mask_hub, hash_hub) {
                (1, 0) => HubState::HubOff,
                (1, 1) => HubState::HubSet,
                (0, _) => HubState::HubUnknownA,
                _ => unreachable!("unexpected match ({} {})", mask_hub, hash_hub)
            },
            hash_init: ma_hash.hash & iter_mask & ma_hash.mask,
            current: 0,
            // the value of the nth page is actually (n - 1)
            num_pages: num_pages - 1,
            mask: ma_hash.mask & iter_mask,
            highest_usable_bit: highest_usable_bit,
            max: iterations,
        }
    }

    fn last_bit(&self) -> u32 {
        (1 << (self.highest_usable_bit))
    }

    // calculates hash without the highest usable
    // bit value, and then increments the current
    // count
    fn calc_hash(&mut self) -> u32 {
        let mut page_id = self.hash_init;

        // bit to read from `current`
        let mut r_cursor = 0u8;

        // bit to write to in `page_id`
        let mut w_cursor = 0u8;

        // replace the ambiguous bits with the bits
        // of current iteration count
        while w_cursor < self.highest_usable_bit {
            // check if bit is ambiguous or not, if so insert into page_id
            // @ the value of w_cursor, from the value of current @ the
            // value of r_cursor & advance the r_cursor
            if ith_bit(w_cursor, self.mask) == 0 {
                page_id |= ith_bit(r_cursor, self.current) << w_cursor;
                r_cursor += 1;
            }
            w_cursor += 1;
        }

        self.current += 1;

        page_id
    }
}


impl Iterator for PageIdIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.current == self.max { return None; }
        println!("{:?}", self);

        // Hub stands for highest usable bit
        match self.state {
            // When it's know the highest set bit is
            HubState::HubOff => Some(self.calc_hash()),

            // When the highest usable bit is known to be 1
            HubState::HubSet => {
                let hash = self.calc_hash() | self.last_bit();
                if hash > self.num_pages {
                    self.state = HubState::HubOff;
                    return Some(hash - self.last_bit());
                }
                else {
                    return Some(hash);
                }
            },

            // When the Highest usable bit is unknown, there are
            // two states, a state where we calculate it with the
            // highest usable state (A) and a state without it (B).
            HubState::HubUnknownA => {
                let last = self.last_bit();
                let hash = self.calc_hash() | last;

                // if our hash is above the number of pages we
                // down grade to the HubOff state, because we'll
                // no longer be using the highest usable bit
                if hash > self.num_pages {
                    self.state = HubState::HubOff;
                    return Some(hash - last);
                }
                else {
                    // note the removal of the highest set bit
                    self.state = HubState::HubUnknownB(hash - last);
                    return Some(hash);
                }
            },
            // this will yield the same result above except with
            // the highest usable set bit removed.
            HubState::HubUnknownB(hash) => {
                self.state = HubState::HubUnknownA;
                return Some(hash);
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::{ PartialHash, FULL_MASK, PageIdIter };
    use query::{ Query };
    use tuple::Tuple;
    use choice_vec::ChoiceVec;
    use rand::random;

    // hashing

    #[test]
    fn hash_mask_correctness() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let PartialHash { hash, mask } = PartialHash::from_query(&query, &c_vec);
        assert_eq!(hash & mask, hash);
        assert_eq!(hash | mask, mask);
    }

    #[test]
    fn hash_when_known_isnt_0() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let ma_hash = PartialHash::from_query(&query, &c_vec);
        assert!(ma_hash.mask != 0);
    }

    #[test]
    fn hash_with_unknown() {
        let query = Query::parse("?,?,?", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let PartialHash { hash, mask } = PartialHash::from_query(&query, &c_vec);
        assert_eq!(hash, 0);
        assert_eq!(mask, 0);
    }

    #[test]
    fn query_hash_same_as_tuple_hash() {
        let c_vec = ChoiceVec::parse("0,0:0,1:1,0:1,1:2,0:2,1", 3).unwrap();
        let tuple = Tuple::parse("a,b,c", 3).unwrap();
        let query = Query::parse("a,b,c", 3).unwrap();
        let ma_hash = PartialHash::from_query(&query, &c_vec);
        assert_eq!(tuple.hash(&c_vec), ma_hash.hash);
    }

    // matching hashed query

    #[test]
    fn hash_matching_zero() {
        let num_hash: u32 = 0;
        let query_hash = PartialHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    #[test]
    fn hash_matching_non_zero() {
        let num_hash = random::<u32>();
        let query_hash = PartialHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    // hash iter

    #[test]
    fn iter_with_no_ambiguities() {
        let mut iter = PageIdIter::new(&PartialHash { hash: 0, mask: FULL_MASK }, 3);
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_yields_correct_values_for_5_pages() {
        // 1 -> 100
        // 2 -> 001
        // 3 -> 010
        // 4 -> 011
        let mut iter = PageIdIter::new(&PartialHash { hash: FULL_MASK, mask: 0b100 }, 5);
        let expected: HashSet<u32> = vec![0b100, 0b1, 0b10, 0b11].into_iter().collect();
        let results : HashSet<u32> = iter.collect();
        assert_eq!(expected, results);
    }

    #[test]
    fn iter_yields_correct_values_for_8_pages() {
        // 1 -> 100
        // 2 -> 101
        // 3 -> 110
        // 4 -> 111
        let mut iter = PageIdIter::new(&PartialHash { hash: FULL_MASK, mask: 0b100 }, 8);
        let expected: HashSet<u32> = vec![0b100, 0b101, 0b110, 0b111].into_iter().collect();
        let results : HashSet<u32> = iter.collect();
        assert_eq!(expected, results);
    }

    #[test]
    fn iter_yields_correct_values_for_8_pages_with_unknown_hub() {
        // 1 -> 101
        // 2 -> 001
        // 3 -> 111
        // 4 -> 011
        let mut iter = PageIdIter::new(&PartialHash { hash: FULL_MASK, mask: 0b001 }, 8);
        let expected: HashSet<u32> = vec![0b01, 0b11, 0b101, 0b111].into_iter().collect();
        let results : HashSet<u32> = iter.collect();
        assert_eq!(expected, results);
    }

    #[test]
    fn iter_single_page() {
        let mut iter = PageIdIter::new(&PartialHash { hash: 1, mask: 1}, 1);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_sp_zero() {
        // 1 -> 00
        // 2 -> 01
        let mut iter = PageIdIter::new(&PartialHash { hash : 0b10, mask: 0b10}, 2);
        let expected: HashSet<u32> = vec![0, 1].into_iter().collect();
        let results : HashSet<u32> = iter.collect();
        assert_eq!(expected, results);
    }

    #[test]
    fn iter_3_pages() {
        let mut iter = PageIdIter::new(&PartialHash { hash : 0b10, mask: 0b10}, 3);
        let expected: HashSet<u32> = vec![1, 2].into_iter().collect();
        let results : HashSet<u32> = iter.collect();
        assert_eq!(expected, results);
    }
}

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
pub struct PageIdIter {
    /// The number of yielded page_ids
    current: u32,
    /// the nominal depth of the relation associated
    /// to this hashes iterator
    num_pages: u32,
    /// Initial hash value, ambiguous bits will be 0
    hash_init: u32,
    /// Used to check which bits are ambiguous
    mask: u32,
    /// The max number of page_ids to yield
    max: u32,
    /// The depth of thet bits used in a page
    hsb: u8,
}

impl PartialHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
    }

    pub fn possible_ids(&self, num_pages: u32) -> PageIdIter {
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
        let hsb = highest_set_bit(num_pages);
        // used to calculate the max number of iterations
        let mut iterations = 1;
        // iter_mask is a mask remove any trailing bits
        // from the iterators mask & init_hash
        let mut iter_mask = 0;

        for i in 0..hsb {
            let position = 1 << i;
            iter_mask |= position;
            if ma_hash.mask & position == 0 {
                iterations <<= 1;
            }
        }

        return PageIdIter {
            hash_init: ma_hash.hash & iter_mask & ma_hash.mask,
            current: 0,
            num_pages: num_pages,
            mask: ma_hash.mask & iter_mask,
            hsb: hsb,
            max: iterations,
        };
    }
}


impl Iterator for PageIdIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        trace!("PageIdIter::next, {:?}", self);
        if self.max == self.current {
            return None;
        }
        if self.num_pages == 1 {
            self.current = self.max;
            return Some(0);
        }
        else {
            let mut page_id = self.hash_init;
            // bit to read from `current`
            let mut r_cursor = 0u8;
            // bit to write to in `page_id`
            let mut w_cursor = 0u8;

            // replace the ambiguous bits with the bits
            // of current iteration count
            while w_cursor < self.hsb {
                // check if bit is ambiguous or not, if so insert into page_id
                // @ the value of w_cursor, from the value of current @ the
                // value of r_cursor & advance the r_cursor
                if ith_bit(w_cursor, self.mask) == 0 {
                    page_id |= ith_bit(r_cursor, self.current) << w_cursor;
                    r_cursor += 1;
                }
                w_cursor += 1;
            }

            if page_id < self.num_pages {
                self.current += 1;
                return Some(page_id);
            }
            else {
                self.current = self.max;
                return None
            }
        }
    }
}


#[cfg(test)]
mod tests {
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
    fn iter_yields_correct_values_for_n7() {
        let mut iter = PageIdIter::new(&PartialHash { hash: FULL_MASK, mask: 0b100 }, 8);
        assert_eq!(iter.next(), Some(0b100));
        assert_eq!(iter.next(), Some(0b101));
        assert_eq!(iter.next(), Some(0b110));
        assert_eq!(iter.next(), Some(0b111));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_yields_correct_values_for_n6() {
        let mut iter = PageIdIter::new(&PartialHash { hash: FULL_MASK, mask: 0b100 }, 7);
        assert_eq!(iter.next(), Some(0b100));
        assert_eq!(iter.next(), Some(0b101));
        assert_eq!(iter.next(), Some(0b110));
        assert_eq!(iter.next(), None);
    }
}

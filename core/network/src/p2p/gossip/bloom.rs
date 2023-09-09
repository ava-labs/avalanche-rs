use std::error::Error;
use probabilistic_collections::bloom::BloomFilter;
use avalanche_types::ids::{Id, LEN};
use crate::p2p::gossip::Gossipable;
use byteorder::{BigEndian, ByteOrder};
use proptest::proptest;
use proptest::prelude::*;

#[derive(Debug)]
pub struct Bloom {
    bloom: BloomFilter::<Hasher>,
    // ToDo Which type here ?
    salt: Id,
}

#[derive(Debug, Hash)]
pub struct Hasher {
    hash: Vec<u8>,
    // ToDo Which type here ?
    salt: Id,
}


impl Bloom {
    pub fn new_bloom_filter(
        max_expected_elements: usize,
        false_positive_probability: f64,
    ) -> Self {
        let salt = random_salt();

        Bloom {
            bloom: BloomFilter::new(max_expected_elements, false_positive_probability),
            salt,
        }
    }


    pub fn add(&mut self, gossipable: impl Gossipable) {
        let id = gossipable.get_id();

        let salted = Hasher {
            hash: id.to_vec(),
            salt: self.salt,
        };

        //ToDo Is this what we want here ?
        self.bloom.insert(&salted)
    }

    pub fn has(&self, gossipable: &impl Gossipable) -> bool {
        let id = gossipable.get_id();

        let salted = Hasher {
            hash: id.to_vec(),
            salt: self.salt,
        };

        //ToDo Is this what we want here ?
        self.bloom.contains(&salted)
    }
}

pub fn reset_bloom_filter_if_needed(
    bloom_filter: &mut Bloom,
    false_positive_probability: f64,
) -> bool {
    if bloom_filter.bloom.estimated_fpp() < false_positive_probability {
        return false;
    }

    let new_bloom_filter = BloomFilter::new(bloom_filter.bloom.len(), false_positive_probability);
    let salt = random_salt();

    bloom_filter.bloom = new_bloom_filter;
    bloom_filter.salt = salt;
    true
}

fn random_salt() -> Id {
    let random_32_bytes = random_manager::secure_bytes(32).unwrap();
    let salt: Id = Id::from_slice(random_32_bytes.as_slice());
    salt
}

impl Hasher {
    pub fn write(&mut self, p: &[u8]) -> Result<usize, std::io::Error> {
        self.hash.extend_from_slice(p);
        Ok(p.len())
    }

    pub fn sum(&mut self, b: &[u8]) -> Vec<u8> {
        self.hash.extend_from_slice(b);
        self.hash.clone()
    }

    pub fn reset(&mut self) {
        self.hash = vec![0; LEN];
    }

    pub fn block_size() -> usize {
        LEN
    }

    pub fn sum64(&self) -> u64 {
        let mut salted = [0u8; LEN];

        for i in 0..std::cmp::min(self.hash.len(), LEN) {
            salted[i] = self.hash[i] ^ self.salt.to_vec().get(i).unwrap();
        }

        BigEndian::read_u64(&salted[0..8])
    }

    pub fn size(&self) -> usize {
        self.hash.len()
    }
}

#[derive(Debug, Clone)]
struct TestTx {
    pub id: Id,
}

impl Gossipable for TestTx {
    fn get_id(&self) -> Id {
        self.id
    }

    fn marshal(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }

    fn unmarshal(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

proptest! {
        #![proptest_config(ProptestConfig {
        cases: 100, // Need 100 successful test cases
        .. ProptestConfig::default()
        })]

        #[test]
        fn test_bloom_filter_refresh(
            false_positive_probability in 0.0..1.0f64,
            txs in proptest::collection::vec(any::<[u8; 32]>(), 0..100) // Will populate txs with 0 to 100 [u8; 32]
        ) {
            let mut bloom_filter = Bloom::new_bloom_filter(10, 0.01);
            let mut expected = vec![];

            for tx in txs {
                let should_reset = reset_bloom_filter_if_needed(&mut bloom_filter, false_positive_probability);
                let test_tx = TestTx { id: Id::from_slice(&tx) };
                if should_reset {
                    expected.clear();
                }

                bloom_filter.add(test_tx.clone());
                expected.push(test_tx.clone());

                for expected_tx in &expected {
                    assert!(bloom_filter.has(expected_tx))
                }
            }
        }
    }


use aes::Aes256;
use blake3::Hasher;
use core::convert::TryInto;
use ctr::cipher::generic_array::GenericArray;
use ctr::cipher::{KeyIvInit, StreamCipher};
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use std::io::Read;

const AES_BLOCK_BYTES: usize = 16;

#[allow(dead_code)]
pub trait Drbg {
    fn name(&self) -> &'static str;
    fn reseed(&mut self, seed: &[u8]);
    fn generate_bits(&mut self, bits: usize) -> BitString;
}

#[derive(Clone)]
pub struct BitString {
    pub bits: usize,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct BitTally {
    pub zeros: u64,
    pub ones: u64,
}

impl BitString {
    pub fn storage_bytes(&self) -> usize {
        self.bytes.len()
    }

    pub fn count_bits(&self) -> BitTally {
        let full_bytes = self.bits / 8;
        let remainder = self.bits % 8;

        let mut ones = 0u64;
        for byte in self.bytes.iter().take(full_bytes) {
            ones += byte.count_ones() as u64;
        }

        if remainder > 0 {
            let byte = self.bytes[full_bytes];
            for i in 0..remainder {
                if ((byte >> (7 - i)) & 1) == 1 {
                    ones += 1;
                }
            }
        }

        BitTally {
            ones,
            zeros: self.bits as u64 - ones,
        }
    }
}

fn derive_material(seed: &[u8], context: &str, length: usize) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(context.as_bytes());
    hasher.update(&(length as u64).to_be_bytes());
    hasher.update(seed);
    let mut reader = hasher.finalize_xof();
    let mut out = vec![0u8; length];
    reader
        .read_exact(&mut out)
        .expect("reading from BLAKE3 XOF should not fail");
    out
}

fn derive_seed(seed: &[u8], context: &str) -> [u8; 32] {
    let material = derive_material(seed, context, 32);
    let mut out = [0u8; 32];
    out.copy_from_slice(&material[..32]);
    out
}

pub struct ChaCha20Drbg {
    rng: ChaCha20Rng,
}

impl ChaCha20Drbg {
    pub fn new(seed: &[u8]) -> Self {
        let derived = derive_seed(seed, "chacha20-drbg");
        Self {
            rng: ChaCha20Rng::from_seed(derived),
        }
    }
}

impl Drbg for ChaCha20Drbg {
    fn name(&self) -> &'static str {
        "ChaCha20 DRBG"
    }

    fn reseed(&mut self, seed: &[u8]) {
        let derived = derive_seed(seed, "chacha20-drbg");
        self.rng = ChaCha20Rng::from_seed(derived);
    }

    fn generate_bits(&mut self, bits: usize) -> BitString {
        let byte_len = (bits + 7) / 8;
        let mut bytes = vec![0u8; byte_len];
        self.rng.fill_bytes(&mut bytes);
        BitString { bits, bytes }
    }
}

type Aes256Ctr = ctr::Ctr128BE<Aes256>;

pub struct AesCtrDrbg {
    key: GenericArray<u8, <Aes256 as aes::cipher::KeySizeUser>::KeySize>,
    counter: u128,
}

impl AesCtrDrbg {
    pub fn new(seed: &[u8]) -> Self {
        let material = derive_material(seed, "aes-ctr-drbg", 48);
        let mut key = GenericArray::default();
        key.copy_from_slice(&material[..32]);
        let counter = u128::from_be_bytes(material[32..48].try_into().unwrap());
        Self { key, counter }
    }
}

impl Drbg for AesCtrDrbg {
    fn name(&self) -> &'static str {
        "AES-256-CTR DRBG"
    }

    fn reseed(&mut self, seed: &[u8]) {
        let material = derive_material(seed, "aes-ctr-drbg", 48);
        self.key.copy_from_slice(&material[..32]);
        self.counter = u128::from_be_bytes(material[32..48].try_into().unwrap());
    }

    fn generate_bits(&mut self, bits: usize) -> BitString {
        let byte_len = (bits + 7) / 8;
        let mut bytes = vec![0u8; byte_len];

        let nonce_bytes = self.counter.to_be_bytes();
        let mut cipher = Aes256Ctr::new(&self.key, &nonce_bytes.into());
        cipher.apply_keystream(&mut bytes);

        let blocks_used = (byte_len + AES_BLOCK_BYTES - 1) / AES_BLOCK_BYTES;
        self.counter = self.counter.wrapping_add(blocks_used as u128);

        BitString { bits, bytes }
    }
}

pub struct Blake3XofDrbg {
    key: [u8; 32],
    counter: u64,
}

impl Blake3XofDrbg {
    pub fn new(seed: &[u8]) -> Self {
        let key = derive_seed(seed, "blake3-xof-drbg");
        Self { key, counter: 0 }
    }
}

impl Drbg for Blake3XofDrbg {
    fn name(&self) -> &'static str {
        "BLAKE3 XOF DRBG"
    }

    fn reseed(&mut self, seed: &[u8]) {
        self.key = derive_seed(seed, "blake3-xof-drbg");
        self.counter = 0;
    }

    fn generate_bits(&mut self, bits: usize) -> BitString {
        let byte_len = (bits + 7) / 8;
        let mut bytes = vec![0u8; byte_len];

        let mut hasher = blake3::Hasher::new_keyed(&self.key);
        hasher.update(&self.counter.to_be_bytes());
        let mut reader = hasher.finalize_xof();
        reader
            .read_exact(&mut bytes)
            .expect("reading from BLAKE3 XOF should not fail");
        self.counter = self.counter.wrapping_add(1);

        BitString { bits, bytes }
    }
}

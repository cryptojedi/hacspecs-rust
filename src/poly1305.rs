// Import hacspec and all needed definitions.
use hacspec::*;
hacspec_imports!();

// Import chacha20
use crate::chacha20;
use crate::chacha20::*;

// Type definitions for use in poly1305.

// These are type aliases for convenience
type Block = [u8; 16];

// These are actual types; fixed-length arrays.
bytes!(Tag, 16);

const BLOCKSIZE: usize = 16;

// Define the Poly1305 field and field elements.
#[field(3fffffffffffffffffffffffffffffffb)]
struct FieldElement;

fn key_gen(key: Key, iv: IV) -> Key {
    let block = chacha20::block(key, 0, iv);
    Key::from_slice(&block[0..32])
}

fn encode_r(r: Block) -> FieldElement {
    let r_uint = u128::from_le_bytes(r);
    let r_uint = r_uint & 0x0ffffffc0ffffffc0ffffffc0fffffffu128;
    FieldElement::from_literal(r_uint)
}

// TODO: to_u128l isn't cool
fn encode(block: Bytes) -> FieldElement {
    let w_elem = FieldElement::from_literal(block.to_u128l());
    let l_elem = FieldElement::pow2(8 * block.len());
    w_elem + l_elem
}

fn poly_inner(m: Bytes, r: FieldElement) -> FieldElement {
    let blocks = m.split(BLOCKSIZE);
    let mut acc = FieldElement::from_literal(0);
    for b in blocks {
        acc = (acc + encode(b)) * r;
    }
    acc
}

pub fn poly(m: Bytes, key: Key) -> Tag {
    let r = to_array(&key[0..BLOCKSIZE]);
    let s = to_array(&key[BLOCKSIZE..2 * BLOCKSIZE]);
    let s_elem = FieldElement::from_literal(u128::from_le_bytes(s));
    let r_elem = encode_r(r);
    let a = poly_inner(m, r_elem);
    let n = a + s_elem;
    let n_bytes = n.to_bytes_le();
    Tag::from_slice(&n_bytes[0..min(16, n_bytes.len())])
}

pub fn poly_mac(m: Bytes, key: Key, iv: IV) -> Tag {
    let mac_key = key_gen(key, iv);
    poly(m, mac_key)
}
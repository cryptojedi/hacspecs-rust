// Import hacspec and all needed definitions.
use hacspec::*;
hacspec_imports!();

// TODO: can we do refinement types somehow?

const BLOCKSIZE: usize = 16;
const IVSIZE: usize = 12;

bytes!(Block, BLOCKSIZE);
bytes!(Word, 4);
bytes!(Key, BLOCKSIZE);
bytes!(Nonce, IVSIZE);
bytes!(SBox, 256);
bytes!(RCon, 11);

bytes!(Bytes144, 144);
bytes!(Bytes176, 176);

type State = [u32; BLOCKSIZE];

const SBOX:SBox = SBox([
    0x63, 0x7C, 0x77, 0x7B, 0xF2, 0x6B, 0x6F, 0xC5, 0x30, 0x01, 0x67, 0x2B, 0xFE, 0xD7, 0xAB, 0x76,
    0xCA, 0x82, 0xC9, 0x7D, 0xFA, 0x59, 0x47, 0xF0, 0xAD, 0xD4, 0xA2, 0xAF, 0x9C, 0xA4, 0x72, 0xC0,
    0xB7, 0xFD, 0x93, 0x26, 0x36, 0x3F, 0xF7, 0xCC, 0x34, 0xA5, 0xE5, 0xF1, 0x71, 0xD8, 0x31, 0x15,
    0x04, 0xC7, 0x23, 0xC3, 0x18, 0x96, 0x05, 0x9A, 0x07, 0x12, 0x80, 0xE2, 0xEB, 0x27, 0xB2, 0x75,
    0x09, 0x83, 0x2C, 0x1A, 0x1B, 0x6E, 0x5A, 0xA0, 0x52, 0x3B, 0xD6, 0xB3, 0x29, 0xE3, 0x2F, 0x84,
    0x53, 0xD1, 0x00, 0xED, 0x20, 0xFC, 0xB1, 0x5B, 0x6A, 0xCB, 0xBE, 0x39, 0x4A, 0x4C, 0x58, 0xCF,
    0xD0, 0xEF, 0xAA, 0xFB, 0x43, 0x4D, 0x33, 0x85, 0x45, 0xF9, 0x02, 0x7F, 0x50, 0x3C, 0x9F, 0xA8,
    0x51, 0xA3, 0x40, 0x8F, 0x92, 0x9D, 0x38, 0xF5, 0xBC, 0xB6, 0xDA, 0x21, 0x10, 0xFF, 0xF3, 0xD2,
    0xCD, 0x0C, 0x13, 0xEC, 0x5F, 0x97, 0x44, 0x17, 0xC4, 0xA7, 0x7E, 0x3D, 0x64, 0x5D, 0x19, 0x73,
    0x60, 0x81, 0x4F, 0xDC, 0x22, 0x2A, 0x90, 0x88, 0x46, 0xEE, 0xB8, 0x14, 0xDE, 0x5E, 0x0B, 0xDB,
    0xE0, 0x32, 0x3A, 0x0A, 0x49, 0x06, 0x24, 0x5C, 0xC2, 0xD3, 0xAC, 0x62, 0x91, 0x95, 0xE4, 0x79,
    0xE7, 0xC8, 0x37, 0x6D, 0x8D, 0xD5, 0x4E, 0xA9, 0x6C, 0x56, 0xF4, 0xEA, 0x65, 0x7A, 0xAE, 0x08,
    0xBA, 0x78, 0x25, 0x2E, 0x1C, 0xA6, 0xB4, 0xC6, 0xE8, 0xDD, 0x74, 0x1F, 0x4B, 0xBD, 0x8B, 0x8A,
    0x70, 0x3E, 0xB5, 0x66, 0x48, 0x03, 0xF6, 0x0E, 0x61, 0x35, 0x57, 0xB9, 0x86, 0xC1, 0x1D, 0x9E,
    0xE1, 0xF8, 0x98, 0x11, 0x69, 0xD9, 0x8E, 0x94, 0x9B, 0x1E, 0x87, 0xE9, 0xCE, 0x55, 0x28, 0xDF,
    0x8C, 0xA1, 0x89, 0x0D, 0xBF, 0xE6, 0x42, 0x68, 0x41, 0x99, 0x2D, 0x0F, 0xB0, 0x54, 0xBB, 0x16
]);

fn subBytes(state: Block) -> Block {
    let mut st = state;
    for i in 0..16 {
        st[i] = SBOX[state[i]];
    }
    st
}

fn shiftRow(i: usize, shift: usize, state: Block) -> Block {
    assert!(i < 4 && shift < 4);
    let mut out = state;
    out[i] = state[i + (4 * (shift % 4))];
    out[i+4] = state[i + (4 * ((shift + 1) % 4))];
    out[i+8] = state[i + (4 * ((shift + 2) % 4))];
    out[i+12] = state[i + (4 * ((shift + 3) % 4))];
    out
}

fn shiftRows(state: Block) -> Block {
    let state = shiftRow(1, 1, state);
    let state = shiftRow(2, 2, state);
    shiftRow(3, 3, state)
}

fn xtime(x: u8) -> u8 {
    let x1 = x << 1;
    let x7 = x >> 7;
    let x71 = x7 & 1;
    let x711b = x71 * 0x1b;
    x1 ^ x711b
}

fn mixColumn(c: usize, state: Block) -> Block {
    assert!(c < 4);
    let i0 = 4 * c;
    let s0 = state[i0];
    let s1 = state[i0+1];
    let s2 = state[i0+2];
    let s3 = state[i0+3];
    let mut st = state;
    let tmp = s0 ^ s1 ^ s2 ^ s3;
    st[i0]   = s0 ^ tmp ^ (xtime (s0 ^ s1));
    st[i0+1] = s1 ^ tmp ^ (xtime (s1 ^ s2));
    st[i0+2] = s2 ^ tmp ^ (xtime (s2 ^ s3));
    st[i0+3] = s3 ^ tmp ^ (xtime (s3 ^ s0));
    st
}

fn mixColumns(state: Block) -> Block {
    let state = mixColumn(0,state);
    let state = mixColumn(1,state);
    let state = mixColumn(2,state);
    mixColumn(3,state)
}

fn addRoundKey(state: Block, key: Key) -> Block {
    let mut out = state;
    for i in 0..16 {
        out[i] ^= key[i];
    }
    out
}

fn aes_enc(state: Block, round_key: Key) -> Block {
    let state = subBytes(state);
    let state = shiftRows(state);
    let state = mixColumns(state);
    addRoundKey(state,round_key)
}

fn aes_enc_last(state: Block, round_key: Key) -> Block {
    let state = subBytes(state);
    let state = shiftRows(state);
    addRoundKey(state,round_key)
}

// TODO: get rid of into
fn rounds(state: Block, key: Bytes144) -> Block {
    let mut out = state;
    for i in 0..9 {
        out = aes_enc(out, key[16*i..16*i+16].into());
    }
    out
}

// TODO: get rid of into
fn block_cipher(input: Block, key: Bytes176) -> Block {
    let k0: Key = key[0..16].into();
    let k: Bytes144 = key[16..10*16].into();
    let kn: Key = key[10*16..11*16].into();
    let state = addRoundKey(input, k0);
    let state = rounds(state, k);
    aes_enc_last(state, kn)
}

// TODO: this and sub_word could be written in a nicer way
fn rotate_word(w: Word) -> Word {
    let mut out = w;
    out[0] = w[1];
    out[1] = w[2];
    out[2] = w[3];
    out[3] = w[0];
    out
}

fn sub_word(w: Word) -> Word {
    let mut out = w;
    out[0] = SBOX[w[0]];
    out[1] = SBOX[w[1]];
    out[2] = SBOX[w[2]];
    out[3] = SBOX[w[3]];
    out
}

const RCON: RCon = RCon([0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36]);

fn aes_keygen_assist(w: Word, rcon: u8) -> Word {
    let k = rotate_word(w);
    let mut k = sub_word(k);
    k[0] ^= rcon;
    k
}

fn key_expansion_word(w0: Word, w1: Word, i: usize) -> Word {
    assert!(i < 44);
    let mut k = w1;
    if i % 4 == 0 {
        k = aes_keygen_assist(k, RCON[i/4]);
    }
    for i in 0..4 {
        k[i] ^= w0[i];
    }
    k
}

// fn key_expansion(key: Block) -> Bytes176 {
//     let mut key_ex = Bytes176::new();
//     key_ex[0..16] = key;
//     let mut i = 0;
//     for j in 0..40 {
//         i = j + 4;
//         key_ex[4*i..4*i+4] = key_expansion_word(key_ex[4*i-16..4*i-12], key_ex[4*i-4..4*i],i);
//     }
//     key_ex
// }

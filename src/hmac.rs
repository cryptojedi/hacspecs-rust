// Import hacspec and all needed definitions.
use hacspec::prelude::*;

use crate::sha2;

const HASH_LEN: usize = sha2::HASH_SIZE;
bytes!(PRK, HASH_LEN);

// HMAC
const BLOCK_LEN: usize = sha2::K_SIZE;
bytes!(Block, BLOCK_LEN);

// H(K XOR opad, H(K XOR ipad, text))
pub fn hmac(k: ByteSeq, txt: ByteSeq) -> PRK {
    let i_pad: Block = Block::from([0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36]);
    let o_pad: Block = Block::from([0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c]);
    
    // Applications that use keys longer than B bytes will first hash the key using H and then use the resultant L byte string as the actual key to HMAC
    let k_block = if k.len() > BLOCK_LEN {
        sha2::hash(k).raw().into()
    } else {
        Block::copy_pad(k)
    };

    let k_ipad = k_block ^ i_pad;
    let k_opad = k_block ^ o_pad;

    // TODO: we need something like append in the lib. Or do we want to stick with pre-allocation?
    let mut h_in = ByteSeq::new_len(BLOCK_LEN + txt.len());
    h_in = h_in.update(0, k_ipad);
    h_in = h_in.update(BLOCK_LEN, txt.clone());
    let h_inner = sha2::hash(h_in);

    let mut h_in = ByteSeq::new_len(BLOCK_LEN + h_inner.len());
    h_in = h_in.update(0, k_opad);
    h_in = h_in.update(BLOCK_LEN, h_inner);
    sha2::hash(h_in).raw().into()
}

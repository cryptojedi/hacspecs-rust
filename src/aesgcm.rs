// Import hacspec and all needed definitions.
use hacspec::prelude::*;

// Import aes and gcm
use crate::aes;
use crate::aes::{aes128_ctr_keyblock, aes128_decrypt, aes128_encrypt, Block};

use crate::gf128::{gmac, Tag, Key};

fn pad_aad_msg(aad: ByteSeq, msg: ByteSeq) -> ByteSeq {
    let laad = aad.len();
    let lmsg = msg.len();
    let pad_aad = if laad % 16 == 0 {
        laad
    } else {
        laad + (16 - (laad % 16))
    };
    let pad_msg = if lmsg % 16 == 0 {
        lmsg
    } else {
        lmsg + (16 - (lmsg % 16))
    };
    let mut padded_msg = ByteSeq::new(pad_aad + pad_msg + 16);
    padded_msg = padded_msg.update(0, aad);
    padded_msg = padded_msg.update(pad_aad, msg);
    padded_msg = padded_msg.update(pad_aad + pad_msg, u64_to_be_bytes(U64(laad as u64) * U64(8)));
    padded_msg = padded_msg.update(pad_aad + pad_msg + 8, u64_to_be_bytes(U64(lmsg as u64) * U64(8)));
    padded_msg
}

pub fn encrypt(key: aes::Key, iv: aes::Nonce, aad: ByteSeq, msg: ByteSeq) -> (ByteSeq, Tag) {
    let iv0 = aes::Nonce::new();

    let mac_key = aes128_ctr_keyblock(key, iv0, U32(0));
    let tag_mix = aes128_ctr_keyblock(key, iv, U32(1));

    let cipher_text = aes128_encrypt(key, iv, U32(2), msg);
    let padded_msg = pad_aad_msg(aad, cipher_text.clone());
    let tag = gmac(padded_msg, Key::copy(mac_key));
    let tag = aes::xor_block(Block::copy(tag), tag_mix);

    (cipher_text, Tag::copy(tag))
}

pub fn decrypt(
    key: aes::Key,
    iv: aes::Nonce,
    aad: ByteSeq,
    cipher_text: ByteSeq,
    tag: Tag,
) -> Result<ByteSeq, String> {
    let iv0 = aes::Nonce::new();

    let mac_key = aes128_ctr_keyblock(key, iv0, U32(0));
    let tag_mix = aes128_ctr_keyblock(key, iv, U32(1));

    let padded_msg = pad_aad_msg(aad, cipher_text.clone());
    let my_tag = gmac(padded_msg, Key::copy(mac_key));
    let my_tag = aes::xor_block(Block::copy(my_tag), tag_mix);

    if my_tag == Block::copy(tag) {
        Ok(aes128_decrypt(key, iv, U32(2), cipher_text))
    } else {
        Err("Mac verification failed".to_string())
    }
}

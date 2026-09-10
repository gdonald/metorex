// The message digests metorex carries, written out here because the whole
// point of a digest is that every implementation agrees on the answer.

/// Pad a message the way MD5 does: a single set bit, zeros, and the bit
/// length as a little-endian 64-bit count.
fn pad_little_endian(message: &[u8]) -> Vec<u8> {
    let mut padded = message.to_vec();
    let bit_length = (message.len() as u64).wrapping_mul(8);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_le_bytes());
    padded
}

/// The same padding, with the bit length written big-endian, which is what
/// the SHA family asks for.
fn pad_big_endian(message: &[u8], block: usize, length_bytes: usize) -> Vec<u8> {
    let mut padded = message.to_vec();
    let bit_length = (message.len() as u128).wrapping_mul(8);
    padded.push(0x80);
    while padded.len() % block != block - length_bytes {
        padded.push(0);
    }
    let counted = bit_length.to_be_bytes();
    padded.extend_from_slice(&counted[16 - length_bytes..]);
    padded
}

const MD5_SHIFTS: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// The MD5 round constants, each the integer part of `abs(sin(i + 1)) * 2^32`.
const MD5_SINES: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// The MD5 digest of a message, sixteen bytes long.
pub(crate) fn md5(message: &[u8]) -> Vec<u8> {
    let mut state: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
    let padded = pad_little_endian(message);
    for chunk in padded.chunks(64) {
        let mut words = [0u32; 16];
        for (index, word) in words.iter_mut().enumerate() {
            let at = index * 4;
            *word = u32::from_le_bytes([chunk[at], chunk[at + 1], chunk[at + 2], chunk[at + 3]]);
        }
        let [mut a, mut b, mut c, mut d] = state;
        for round in 0..64 {
            let (mixed, index) = match round / 16 {
                0 => ((b & c) | (!b & d), round),
                1 => ((d & b) | (!d & c), (5 * round + 1) % 16),
                2 => (b ^ c ^ d, (3 * round + 5) % 16),
                _ => (c ^ (b | !d), (7 * round) % 16),
            };
            let shifted = a
                .wrapping_add(mixed)
                .wrapping_add(MD5_SINES[round])
                .wrapping_add(words[index]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(shifted.rotate_left(MD5_SHIFTS[round]));
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }
    state.iter().flat_map(|word| word.to_le_bytes()).collect()
}

/// The SHA-1 digest of a message, twenty bytes long.
pub(crate) fn sha1(message: &[u8]) -> Vec<u8> {
    let mut state: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    let padded = pad_big_endian(message, 64, 8);
    for chunk in padded.chunks(64) {
        let mut schedule = [0u32; 80];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            let at = index * 4;
            *word = u32::from_be_bytes([chunk[at], chunk[at + 1], chunk[at + 2], chunk[at + 3]]);
        }
        for index in 16..80 {
            schedule[index] = (schedule[index - 3]
                ^ schedule[index - 8]
                ^ schedule[index - 14]
                ^ schedule[index - 16])
                .rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = state;
        for (round, word) in schedule.iter().enumerate() {
            let (mixed, constant) = match round / 20 {
                0 => ((b & c) | (!b & d), 0x5a827999),
                1 => (b ^ c ^ d, 0x6ed9eba1),
                2 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let shifted = a
                .rotate_left(5)
                .wrapping_add(mixed)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = shifted;
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e]) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().flat_map(|word| word.to_be_bytes()).collect()
}

/// The SHA-256 round constants, the fractional parts of the cube roots of
/// the first sixty-four primes.
const SHA256_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The SHA-256 digest of a message, thirty-two bytes long.
pub(crate) fn sha256(message: &[u8]) -> Vec<u8> {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let padded = pad_big_endian(message, 64, 8);
    for chunk in padded.chunks(64) {
        let mut schedule = [0u32; 64];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            let at = index * 4;
            *word = u32::from_be_bytes([chunk[at], chunk[at + 1], chunk[at + 2], chunk[at + 3]]);
        }
        for index in 16..64 {
            let low = schedule[index - 15];
            let high = schedule[index - 2];
            let spread_low = low.rotate_right(7) ^ low.rotate_right(18) ^ (low >> 3);
            let spread_high = high.rotate_right(17) ^ high.rotate_right(19) ^ (high >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(spread_low)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(spread_high);
        }
        let mut working = state;
        for (round, word) in schedule.iter().enumerate() {
            let [a, b, c, d, e, f, g, h] = working;
            let spread_e = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let chosen = (e & f) ^ (!e & g);
            let first = h
                .wrapping_add(spread_e)
                .wrapping_add(chosen)
                .wrapping_add(SHA256_CONSTANTS[round])
                .wrapping_add(*word);
            let spread_a = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let second = spread_a.wrapping_add(majority);
            working = [
                first.wrapping_add(second),
                a,
                b,
                c,
                d.wrapping_add(first),
                e,
                f,
                g,
            ];
        }
        for (slot, value) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().flat_map(|word| word.to_be_bytes()).collect()
}

/// The SHA-512 round constants, the fractional parts of the cube roots of
/// the first eighty primes.
const SHA512_CONSTANTS: [u64; 80] = [
    0x428a2f98d728ae22,
    0x7137449123ef65cd,
    0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc,
    0x3956c25bf348b538,
    0x59f111f1b605d019,
    0x923f82a4af194f9b,
    0xab1c5ed5da6d8118,
    0xd807aa98a3030242,
    0x12835b0145706fbe,
    0x243185be4ee4b28c,
    0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,
    0x80deb1fe3b1696b1,
    0x9bdc06a725c71235,
    0xc19bf174cf692694,
    0xe49b69c19ef14ad2,
    0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5,
    0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,
    0x4a7484aa6ea6e483,
    0x5cb0a9dcbd41fbd4,
    0x76f988da831153b5,
    0x983e5152ee66dfab,
    0xa831c66d2db43210,
    0xb00327c898fb213f,
    0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,
    0xd5a79147930aa725,
    0x06ca6351e003826f,
    0x142929670a0e6e70,
    0x27b70a8546d22ffc,
    0x2e1b21385c26c926,
    0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df,
    0x650a73548baf63de,
    0x766a0abb3c77b2a8,
    0x81c2c92e47edaee6,
    0x92722c851482353b,
    0xa2bfe8a14cf10364,
    0xa81a664bbc423001,
    0xc24b8b70d0f89791,
    0xc76c51a30654be30,
    0xd192e819d6ef5218,
    0xd69906245565a910,
    0xf40e35855771202a,
    0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,
    0x1e376c085141ab53,
    0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,
    0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,
    0x78a5636f43172f60,
    0x84c87814a1f0ab72,
    0x8cc702081a6439ec,
    0x90befffa23631e28,
    0xa4506cebde82bde9,
    0xbef9a3f7b2c67915,
    0xc67178f2e372532b,
    0xca273eceea26619c,
    0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e,
    0xf57d4f7fee6ed178,
    0x06f067aa72176fba,
    0x0a637dc5a2c898a6,
    0x113f9804bef90dae,
    0x1b710b35131c471b,
    0x28db77f523047d84,
    0x32caab7b40c72493,
    0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,
    0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec,
    0x6c44198c4a475817,
];

/// One pass of the SHA-512 compression function over a padded message,
/// answering the eight words of state it leaves behind.
fn sha512_state(message: &[u8], mut state: [u64; 8]) -> [u64; 8] {
    let padded = pad_big_endian(message, 128, 16);
    for chunk in padded.chunks(128) {
        let mut schedule = [0u64; 80];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            let at = index * 8;
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&chunk[at..at + 8]);
            *word = u64::from_be_bytes(bytes);
        }
        for index in 16..80 {
            let low = schedule[index - 15];
            let high = schedule[index - 2];
            let spread_low = low.rotate_right(1) ^ low.rotate_right(8) ^ (low >> 7);
            let spread_high = high.rotate_right(19) ^ high.rotate_right(61) ^ (high >> 6);
            schedule[index] = schedule[index - 16]
                .wrapping_add(spread_low)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(spread_high);
        }
        let mut working = state;
        for (round, word) in schedule.iter().enumerate() {
            let [a, b, c, d, e, f, g, h] = working;
            let spread_e = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let chosen = (e & f) ^ (!e & g);
            let first = h
                .wrapping_add(spread_e)
                .wrapping_add(chosen)
                .wrapping_add(SHA512_CONSTANTS[round])
                .wrapping_add(*word);
            let spread_a = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let second = spread_a.wrapping_add(majority);
            working = [
                first.wrapping_add(second),
                a,
                b,
                c,
                d.wrapping_add(first),
                e,
                f,
                g,
            ];
        }
        for (slot, value) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(value);
        }
    }
    state
}

/// The SHA-512 digest of a message, sixty-four bytes long.
pub(crate) fn sha512(message: &[u8]) -> Vec<u8> {
    let state = sha512_state(
        message,
        [
            0x6a09e667f3bcc908,
            0xbb67ae8584caa73b,
            0x3c6ef372fe94f82b,
            0xa54ff53a5f1d36f1,
            0x510e527fade682d1,
            0x9b05688c2b3e6c1f,
            0x1f83d9abfb41bd6b,
            0x5be0cd19137e2179,
        ],
    );
    state.iter().flat_map(|word| word.to_be_bytes()).collect()
}

/// The SHA-384 digest of a message: SHA-512 started from its own eight words
/// and cut to the first forty-eight bytes.
pub(crate) fn sha384(message: &[u8]) -> Vec<u8> {
    let state = sha512_state(
        message,
        [
            0xcbbb9d5dc1059ed8,
            0x629a292a367cd507,
            0x9159015a3070dd17,
            0x152fecd8f70e5939,
            0x67332667ffc00b31,
            0x8eb44a8768581511,
            0xdb0c2e0d64f98fa7,
            0x47b5481dbefa4fa4,
        ],
    );
    state
        .iter()
        .flat_map(|word| word.to_be_bytes())
        .take(48)
        .collect()
}

/// The digest a named algorithm gives, or None for a name metorex has no
/// implementation of.
pub(crate) fn digest_named(algorithm: &str, message: &[u8]) -> Option<Vec<u8>> {
    match algorithm {
        "MD5" => Some(md5(message)),
        "SHA1" => Some(sha1(message)),
        "SHA256" => Some(sha256(message)),
        "SHA384" => Some(sha384(message)),
        "SHA512" => Some(sha512(message)),
        _ => None,
    }
}

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// `Digest.__digest__(algorithm, message)` — the raw digest bytes, held
    /// one to a character the way `Array#pack` holds them.
    pub(crate) fn compute_digest(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(Object::String(algorithm)), Some(Object::String(message))) =
            (arguments.first(), arguments.get(1))
        else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Exact(2),
                arguments.len(),
                position,
            ));
        };
        let bytes = super::pack_format::string_to_bytes(&message.as_str());
        match digest_named(&algorithm.as_str(), &bytes) {
            Some(digested) => Ok(super::pack_format::bytes_to_string(&digested)),
            None => Err(MetorexError::runtime_error(
                format!("unknown digest algorithm {}", algorithm.as_str()),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }

    /// `Digest.__pbkdf2__(algorithm, pass, salt, rounds, length)` — the
    /// derived key, computed here because the rounds number in the tens of
    /// thousands and each one is a pair of digests.
    pub(crate) fn compute_pbkdf2(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (
            Some(Object::String(algorithm)),
            Some(Object::String(pass)),
            Some(Object::String(salt)),
            Some(Object::Int(rounds)),
            Some(Object::Int(length)),
        ) = (
            arguments.first(),
            arguments.get(1),
            arguments.get(2),
            arguments.get(3),
            arguments.get(4),
        )
        else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::Exact(5),
                arguments.len(),
                position,
            ));
        };
        let derived = pbkdf2(
            &algorithm.as_str(),
            &super::pack_format::string_to_bytes(&pass.as_str()),
            &super::pack_format::string_to_bytes(&salt.as_str()),
            (*rounds).max(1) as u32,
            (*length).max(0) as usize,
        );
        match derived {
            Some(held) => Ok(super::pack_format::bytes_to_string(&held)),
            None => Err(MetorexError::runtime_error(
                format!("unknown digest algorithm {}", algorithm.as_str()),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}

/// The keyed digest: the key brought to one block, then the message digested
/// once inside a padded block and once outside it.
pub(crate) fn hmac(algorithm: &str, key: &[u8], message: &[u8]) -> Option<Vec<u8>> {
    let block = match algorithm {
        "MD5" | "SHA1" | "SHA256" => 64,
        "SHA384" | "SHA512" => 128,
        _ => return None,
    };
    let mut shortened = if key.len() > block {
        digest_named(algorithm, key)?
    } else {
        key.to_vec()
    };
    shortened.resize(block, 0);
    let inner: Vec<u8> = shortened.iter().map(|byte| byte ^ 0x36).collect();
    let outer: Vec<u8> = shortened.iter().map(|byte| byte ^ 0x5c).collect();
    let mut first = inner;
    first.extend_from_slice(message);
    let once = digest_named(algorithm, &first)?;
    let mut second = outer;
    second.extend_from_slice(&once);
    digest_named(algorithm, &second)
}

/// A key derived from a password by keying it against a salt over and over,
/// so that guessing the password one try at a time takes as many rounds.
pub(crate) fn pbkdf2(
    algorithm: &str,
    pass: &[u8],
    salt: &[u8],
    rounds: u32,
    length: usize,
) -> Option<Vec<u8>> {
    let width = digest_named(algorithm, &[])?.len();
    let mut out = Vec::with_capacity(length);
    let mut index = 1u32;
    while out.len() < length {
        let mut seeded = salt.to_vec();
        seeded.extend_from_slice(&index.to_be_bytes());
        let mut held = hmac(algorithm, pass, &seeded)?;
        let mut folded = held.clone();
        for _ in 1..rounds {
            held = hmac(algorithm, pass, &held)?;
            for (slot, byte) in folded.iter_mut().zip(&held) {
                *slot ^= byte;
            }
        }
        out.extend_from_slice(&folded);
        index += 1;
        if width == 0 {
            return None;
        }
    }
    out.truncate(length);
    Some(out)
}

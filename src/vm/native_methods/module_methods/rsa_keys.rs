// Generating an RSA key: two random primes of half the modulus's size, and
// the exponents and coefficient a private key carries alongside them.

use super::*;
use num_bigint::BigUint;

/// The primes below 2000, which a candidate is divided by before the slower
/// probabilistic test is run on it.
fn small_primes() -> Vec<u32> {
    let mut sieve = vec![true; 2000];
    let mut primes = Vec::new();
    for value in 2..2000 {
        if sieve[value] {
            primes.push(value as u32);
            let mut multiple = value * value;
            while multiple < 2000 {
                sieve[multiple] = false;
                multiple += value;
            }
        }
    }
    primes
}

/// Random bytes from the operating system.
fn random_bytes(count: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read as _;
    let mut bytes = vec![0u8; count];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// A random number below `limit` and at least 2.
fn random_below(limit: &BigUint) -> std::io::Result<BigUint> {
    let width = limit.to_bytes_be().len();
    let two = BigUint::from(2u32);
    loop {
        let candidate = BigUint::from_bytes_be(&random_bytes(width)?) % limit;
        if candidate >= two {
            return Ok(candidate);
        }
    }
}

/// Miller-Rabin with `rounds` random bases.
fn probably_prime(candidate: &BigUint, rounds: usize) -> std::io::Result<bool> {
    let one = BigUint::from(1u32);
    let less_one = candidate - &one;
    let mut odd = less_one.clone();
    let mut twos = 0;
    while !odd.bit(0) {
        odd >>= 1;
        twos += 1;
    }
    'rounds: for _ in 0..rounds {
        let mut held = random_below(&less_one)?.modpow(&odd, candidate);
        if held == one || held == less_one {
            continue;
        }
        for _ in 1..twos {
            held = held.modpow(&BigUint::from(2u32), candidate);
            if held == less_one {
                continue 'rounds;
            }
        }
        return Ok(false);
    }
    Ok(true)
}

/// A random prime of exactly `bits` bits whose top two bits are set, so
/// two of them multiply to a modulus of twice the size, and for which
/// `exponent` has an inverse.
fn random_prime(bits: usize, exponent: &BigUint, primes: &[u32]) -> std::io::Result<BigUint> {
    let one = BigUint::from(1u32);
    loop {
        let mut candidate = BigUint::from_bytes_be(&random_bytes(bits.div_ceil(8))?);
        candidate >>= candidate.bits().saturating_sub(bits as u64);
        candidate.set_bit(bits as u64 - 1, true);
        candidate.set_bit(bits as u64 - 2, true);
        candidate.set_bit(0, true);
        if primes
            .iter()
            .any(|small| &candidate % small == BigUint::ZERO && candidate != BigUint::from(*small))
        {
            continue;
        }
        if num_integer_gcd(&(&candidate - &one), exponent) != one {
            continue;
        }
        if probably_prime(&candidate, 16)? {
            return Ok(candidate);
        }
    }
}

fn num_integer_gcd(left: &BigUint, right: &BigUint) -> BigUint {
    let (mut first, mut second) = (left.clone(), right.clone());
    while second != BigUint::ZERO {
        let rest = &first % &second;
        first = second;
        second = rest;
    }
    first
}

impl VirtualMachine {
    /// `OpenSSL.__rsa_generate__(bits, exponent)`: a new key as its
    /// modulus, public and private exponents, the two primes, the private
    /// exponent reduced by each prime less one, and the inverse of the
    /// second prime modulo the first.
    pub(crate) fn rsa_generate(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(Object::Int(bits)), Some(exponent)) = (arguments.first(), arguments.get(1))
        else {
            return Err(simple_exception(
                "ArgumentError",
                "key size and exponent required",
                position,
            ));
        };
        let exponent = exponent
            .as_big_integer()
            .and_then(|held| held.to_biguint())
            .ok_or_else(|| simple_exception("ArgumentError", "invalid exponent", position))?;
        // An even exponent shares a factor with every prime less one, so no
        // prime would ever be found for it.
        if !exponent.bit(0) || exponent < BigUint::from(3u32) || *bits < 16 {
            return Err(simple_exception(
                "ArgumentError",
                "invalid key size or exponent",
                position,
            ));
        }
        let failed = |problem: std::io::Error| {
            simple_exception("OpenSSL::PKey::RSAError", &problem.to_string(), position)
        };
        let primes = small_primes();
        let half = (*bits as usize).div_ceil(2);
        let one = BigUint::from(1u32);
        let (first, second) = loop {
            let first = random_prime(half, &exponent, &primes).map_err(failed)?;
            let second = random_prime(*bits as usize - half, &exponent, &primes).map_err(failed)?;
            if first != second {
                break if first > second {
                    (first, second)
                } else {
                    (second, first)
                };
            }
        };
        let modulus = &first * &second;
        let first_less = &first - &one;
        let second_less = &second - &one;
        let lcm = &first_less * &second_less / num_integer_gcd(&first_less, &second_less);
        let private = exponent
            .modinv(&lcm)
            .ok_or_else(|| simple_exception("OpenSSL::PKey::RSAError", "no inverse", position))?;
        let first_exponent = &private % &first_less;
        let second_exponent = &private % &second_less;
        let coefficient = second
            .modinv(&first)
            .ok_or_else(|| simple_exception("OpenSSL::PKey::RSAError", "no inverse", position))?;
        let as_integer = |value: BigUint| Object::integer(num_bigint::BigInt::from(value));
        Ok(Object::array(vec![
            as_integer(modulus),
            as_integer(exponent),
            as_integer(private),
            as_integer(first),
            as_integer(second),
            as_integer(first_exponent),
            as_integer(second_exponent),
            as_integer(coefficient),
        ]))
    }
}

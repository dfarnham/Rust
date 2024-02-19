use rand::Rng;
use std::env;
use std::error::Error;
use std::io::{self, Write};

// https://en.wikipedia.org/wiki/Mersenne_Twister

const W: u32 = 32; // word size (number of bits)
const N: u32 = 624; // degree of recurrence
const M: u32 = 397; // middle word, an offset used in the recurrence relation defining the series x, 1 ≤ m < n
const A: u32 = 0x9908B0DF; // coefficients of the rational normal form twist matrix
const U: u32 = 0xB;
const S: u32 = 0x7;
const B: u32 = 0x9D2C5680;
const T: u32 = 0xF;
const C: u32 = 0xEFC60000;
const L: u32 = 0x12;
const F: u32 = 1812433253;
const LOWER_MASK: u32 = 0x7FFFFFFF;
const UPPER_MASK: u32 = !LOWER_MASK;

fn prng_mt19937(count: usize, seed: u32) -> Vec<u32> {
    let mut mt = [0_u32; N as usize];
    let mut idx = N;
    let mut results = vec![];

    // seed
    mt[0] = seed;

    // initialize
    for i in 1..N as usize {
        mt[i] = F * (mt[i - 1] ^ (mt[i - 1] >> (W - 2))) + i as u32;
    }

    // twist
    let twist = |mt: &mut [u32]| {
        for i in 0..N {
            let x = (mt[i as usize] & UPPER_MASK) + (mt[((i + 1) % N) as usize] & LOWER_MASK);
            let t = match x % 2 == 0 {
                true => x >> 1,
                false => (x >> 1) ^ A,
            };
            mt[i as usize] = mt[((i + M) % N) as usize] ^ t;
        }
    };

    // temper
    let temper = |y: u32| {
        let mut y = y;
        y ^= y >> U;
        y ^= (y << S) & B;
        y ^= (y << T) & C;
        y ^ y >> L
    };

    for _ in 0..count {
        if idx >= N {
            twist(&mut mt);
            idx = 0;
        }
        results.push(temper(mt[idx as usize]));
        idx += 1;
    }
    results
}

fn main() -> Result<(), Box<dyn Error>> {
    // behave like a typical unix utility
    general::reset_sigpipe()?;
    let mut stdout = io::stdout().lock();

    // Usage: mt19937 [count] [seed]
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1].starts_with("-h") || args[1].starts_with("--h")) {
        writeln!(stdout, "Usage: mt19937 [count] [seed]")?;
        return Ok(());
    }

    let count = if args.len() > 1 { args[1].parse::<usize>()? } else { 10 };
    let seed = if args.len() > 2 {
        args[2].parse::<u32>()?
    } else {
        rand::thread_rng().gen_range(0..u32::MAX)
    };

    for r in prng_mt19937(count, seed) {
        writeln!(stdout, "{r}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_10_seed_1() {
        assert_eq!(
            prng_mt19937(10, 1),
            [
                1791095845, 4282876139, 3093770124, 4005303368, 491263, 550290313, 1298508491, 4290846341, 630311759,
                1013994432,
            ]
        );
    }
}

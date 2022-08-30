use rand::seq::SliceRandom;
use rand::SeedableRng;

pub const N: usize = 26;

const FROM_SYMBOL: char = 'A';
const TO_SYMBOL: char = 'Z';

pub fn to_index(c: char) -> Option<usize> {
    let base = u32::from(FROM_SYMBOL) as usize;
    let ci = u32::from(c) as usize;
    if ci < base || ci >= (base + N) {
        None
    } else {
        Some(ci - base)
    }
}

pub fn generate(k: u8, seed: Option<u64>) -> Vec<Vec<char>> {
    partitions(symbols(), k, seed)
}

fn symbols() -> Vec<char> {
    (FROM_SYMBOL..=TO_SYMBOL).into_iter().collect::<Vec<char>>()
}

fn partitions(symbols: Vec<char>, k: u8, seed: Option<u64>) -> Vec<Vec<char>> {
    let k = k as usize;
    let sn = symbols.len();
    if k == 1 {
        return vec![symbols];
    } else if k == sn {
        return symbols.into_iter().map(|x| vec![x]).collect();
    }

    let mut symbols = symbols.clone();
    let mut rng: rand::rngs::StdRng = match seed {
        Some(s) => SeedableRng::seed_from_u64(s),
        None => SeedableRng::from_entropy(),
    };

    symbols.shuffle(&mut rng);

    let mut splits = rand::seq::index::sample(&mut rng, sn - 1, k - 1)
        .into_iter()
        .map(|x| x + 1)
        .collect::<Vec<usize>>();
    splits.sort();

    let mut result = Vec::with_capacity(k);
    let mut last = 0;
    for split in splits {
        result.push(symbols[last..split].to_vec());
        last = split;
    }
    result.push(symbols[last..].to_vec());

    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_generate_returns_requested_number_of_partitions() {
        for i in 1..26usize {
            let partitions = generate(i as u8, None);

            assert_eq!(
                partitions.len(),
                i,
                "expect number of partitions to be equal requested number: {}; but got: {}",
                i,
                partitions.len()
            );

            let total_symbols: usize = partitions.iter().map(|x| x.len()).sum();

            assert_eq!(total_symbols, symbols().len(),
                       "expect total number of symbols across all partitions to be equal number of available symbols: {}; got {}",
                       symbols().len(), total_symbols);
        }
    }
}

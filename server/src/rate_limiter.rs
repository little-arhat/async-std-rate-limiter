use core::num::NonZeroU8;

use governor::nanos::Nanos;
use governor::{clock, middleware, state, Quota};

pub type RateLimiter<const N: usize> =
    governor::RateLimiter<usize, ArrayStore<N>, clock::DefaultClock, middleware::NoOpMiddleware>;

const NON_KEY: state::direct::NotKeyed = state::direct::NotKeyed::NonKey;

pub fn init<const N: usize>(partitions: Vec<Vec<usize>>, limit: NonZeroU8) -> RateLimiter<N> {
    let l = limit.into();
    let quota = Quota::per_second(l);

    let mut ixs = [0usize; N];
    for (index, partition) in partitions.into_iter().enumerate() {
        for symbol_index in partition {
            ixs[symbol_index] = index;
        }
    }

    let clock = clock::DefaultClock::default();
    let state = ArrayStore::new(ixs);

    governor::RateLimiter::new(quota, state, &clock)
}

pub struct ArrayStore<const N: usize> {
    states: Vec<state::InMemoryState>,
    mapping: [usize; N],
}

impl<const N: usize> ArrayStore<N> {
    fn new(mapping: [usize; N]) -> Self {
        let max_ix = mapping.into_iter().max().unwrap_or_default();
        ArrayStore {
            states: (0..=max_ix)
                .map(|_| state::InMemoryState::default())
                .collect(),
            mapping,
        }
    }
}

impl<const N: usize> state::StateStore for ArrayStore<N> {
    type Key = usize;

    fn measure_and_replace<T, F, E>(&self, key: &Self::Key, f: F) -> Result<T, E>
    where
        F: Fn(Option<Nanos>) -> Result<(T, Nanos), E>,
    {
        let ix = self.mapping[*key];
        self.states[ix].measure_and_replace(&NON_KEY, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_rate_limiter_respects_group_configuration() {
        let partitions = vec![vec![0, 1], vec![2, 3]];
        let limit = NonZeroU8::new(1).unwrap();
        let rl = init::<4>(partitions, limit);

        let r0 = rl.check_key(&0).is_ok();
        assert_eq!(r0, true, "Expect query for 0 to succeed");
        let r1 = rl.check_key(&1).is_err();
        assert_eq!(r1, true, "Expect query for 1 to fail");

        let r2 = rl.check_key(&2).is_ok();
        assert_eq!(r2, true, "Expect query for 2 to succeed");
        let r3 = rl.check_key(&3).is_err();
        assert_eq!(r3, true, "Expect query for 3 to fail");
    }

    // testing time is out of scope: we rely on governor being tested
}

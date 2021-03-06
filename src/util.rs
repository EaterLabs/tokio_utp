use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::time::Duration;

pub fn as_ms(duration: Duration) -> u64 {
    // Lets just limit to 30 seconds
    if duration.as_secs() > 30 {
        30_000
    } else {
        let sub_secs = duration.subsec_nanos() / NANOS_PER_MS;
        duration.as_secs() * 1000 + u64::from(sub_secs)
    }
}

/// Wrapping less than comparison.
/// E.g. `-10 as u32 < 10`, where `-10 as u32` actually is `4294967276`.
pub fn wrapping_lt(lhs: u32, rhs: u32, mask: u32) -> bool {
    let dist_dn = lhs.wrapping_sub(rhs) & mask;
    let dist_up = rhs.wrapping_sub(lhs) & mask;
    dist_up < dist_dn
}

const MICROS_PER_SEC: u32 = 1_000_000;
const NANOS_PER_MS: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;

pub fn as_wrapping_micros(duration: Duration) -> u32 {
    // Wrapping is OK
    let mut ret = duration.as_secs().wrapping_mul(u64::from(MICROS_PER_SEC)) as u32;
    // TODO(povilas): what if this addition overflows? V
    ret += duration.subsec_nanos() / NANOS_PER_MICRO;
    ret
}

/// Safely generates two sequential connection identifiers.
///
/// This avoids an overflow when the generated receiver identifier is the largest
/// representable value in u16 and it is incremented to yield the corresponding sender
/// identifier.
pub fn generate_sequential_identifiers() -> (u16, u16) {
    let id: u16 = rand();

    if id.checked_add(1).is_some() {
        (id, id + 1)
    } else {
        (id - 1, id)
    }
}

#[cfg(test)]
pub fn random_vec(num_bytes: usize) -> Vec<u8> {
    use rand::RngCore;

    let mut ret = Vec::with_capacity(num_bytes);
    unsafe { ret.set_len(num_bytes) };
    THREAD_RNG.with(|r| r.borrow_mut().fill_bytes(&mut ret[..]));
    ret
}

#[cfg(not(test))]
pub fn rand<T>() -> T
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    rng.gen::<T>()
}

#[cfg(test)]
pub use self::test::{rand, reset_rand, THREAD_RNG};

#[cfg(test)]
mod test {
    use super::*;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::cell::RefCell;

    thread_local!(pub static THREAD_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy()));

    pub fn rand<T>() -> T
    where
        Standard: Distribution<T>,
    {
        THREAD_RNG.with(|t| t.borrow_mut().gen::<T>())
    }

    #[cfg(test)]
    pub fn reset_rand() {
        THREAD_RNG.with(|t| *t.borrow_mut() = SmallRng::from_entropy());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod as_wrapping_micros {
        use super::*;

        #[test]
        fn it_returns_given_duration_in_microseconds() {
            let duration = Duration::new(2, 5000);

            let micros = as_wrapping_micros(duration);

            assert_eq!(micros, 2_000_005);
        }

        #[test]
        fn when_microseconds_overflows_it_returns_wrapped_value() {
            let elapsed = Duration::new(4295, 0);

            let micros = as_wrapping_micros(elapsed);

            assert_eq!(micros, 32_704);
        }
    }

    mod wrapping_lt {
        use super::*;

        #[test]
        fn it_compares_two_numbers_as_if_they_were_signed() {
            let a: u32 = -10i32 as u32;
            let b: u32 = 10;
            let less = wrapping_lt(a, b, 0xFF_FF_FF_FF);

            assert!(less);
        }
    }
}

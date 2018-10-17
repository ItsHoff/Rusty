use std::ops::{AddAssign, DivAssign};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct SumAccumulator<T>
    where T: Zero + AddAssign {
    pub val: T
}

impl<T> SumAccumulator<T> where T: AddAssign + Zero {
    pub fn new() -> Self {
        SumAccumulator { val: T::zero() }
    }

    pub fn add_self(&mut self, other: Self) {
        self.val += other.val;
    }

    pub fn add_val(&mut self, val: T) {
        self.val += val;
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for Duration {
    fn zero() -> Duration {
        Duration::new(0, 0)
    }
}

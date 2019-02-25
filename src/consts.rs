use crate::float::*;

#[cfg(not(feature = "single_precision"))]
pub use self::double::*;
#[cfg(feature = "single_precision")]
pub use self::single::*;

#[cfg(not(feature = "single_precision"))]
mod double {
    use super::*;

    pub const EPSILON: Float = 1e-10;
    pub const INFINITY: Float = std::f64::INFINITY;
    pub const MAX: Float = std::f64::MAX;
    pub const MIN: Float = std::f64::MIN;
    pub const PI: Float = std::f64::consts::PI;
}

#[cfg(feature = "single_precision")]
mod single {
    use super::*;

    pub const EPSILON: Float = 1e-5;
    pub const INFINITY: Float = std::f32::INFINITY;
    pub const MAX: Float = std::f32::MAX;
    pub const MIN: Float = std::f32::MIN;
    pub const PI: Float = std::f32::consts::PI;
}

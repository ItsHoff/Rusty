use crate::float::*;

#[cfg(not(feature = "single_precision"))]
pub use self::double::*;
#[cfg(feature = "single_precision")]
pub use self::single::*;

#[cfg(not(feature = "single_precision"))]
mod double {
    use super::*;

    pub const EPSILON: Float = 1e-10;
    #[allow(dead_code)]
    pub const MACHINE_EPSILON: Float = f64::EPSILON / 2.0;
    pub const INFINITY: Float = f64::INFINITY;
    pub const MAX: Float = f64::MAX;
    pub const MIN: Float = f64::MIN;
    pub const PI: Float = std::f64::consts::PI;
}

#[cfg(feature = "single_precision")]
mod single {
    use super::*;

    pub const EPSILON: Float = 1e-5;
    #[allow(dead_code)]
    pub const MACHINE_EPSILON: Float = f32::EPSILON / 2.0;
    pub const INFINITY: Float = f32::INFINITY;
    pub const MAX: Float = f32::MAX;
    pub const MIN: Float = f32::MIN;
    pub const PI: Float = std::f32::consts::PI;
}

use std::ops::{Add, Div, Mul, Sub};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Serializer};

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, Serialize)]
pub struct Time(pub f64);

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.serialize_f64(self.0)
    }
}
impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Eq for Time {

}


impl Time {
    pub(crate) const MAX: Time = Time(f64::MAX);
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
}

impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Time(self.0 - rhs.0)
    }
}

impl Add<f64> for Time {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        Time(self.0.add(rhs))
    }
}

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Time) -> Self::Output {
        Time(self.0 + rhs.0)
    }
}

impl Div<f64> for Time {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Time(self.0 / rhs)
    }
}

impl Mul<Time> for Time {
    type Output = Self;

    fn mul(self, rhs: Time) -> Self::Output {
        Time(self.0 * rhs.0)
    }
}

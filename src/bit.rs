use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Bit {
    Off,
    On,
}

impl Display for Bit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Bit::Off => '0',
                Bit::On => '1',
            }
        )
    }
}

impl Add<Bit> for Bit {
    type Output = Bit;

    fn add(self, rhs: Bit) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) | (Bit::On, Bit::On) => Bit::Off,
            (Bit::Off, Bit::On) | (Bit::On, Bit::Off) => Bit::On,
        }
    }
}

impl AddAssign<Bit> for Bit {
    fn add_assign(&mut self, rhs: Bit) {
        *self = *self + rhs;
    }
}

impl Mul<Bit> for Bit {
    type Output = Bit;

    fn mul(self, rhs: Bit) -> Self::Output {
        match (self, rhs) {
            (Bit::On, Bit::On) => Bit::On,
            _ => Bit::Off,
        }
    }
}

impl MulAssign<Bit> for Bit {
    fn mul_assign(&mut self, rhs: Bit) {
        *self = *self * rhs;
    }
}

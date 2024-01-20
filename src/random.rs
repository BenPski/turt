use core::result::Result;

use rand::{self, distributions::{Distribution, Standard, Uniform}};

const GAP_SIZE: u32 = 0xDFFF - 0xD800 + 1;

#[derive(Debug)]
pub struct Digit {
    pub val: char,
}

#[derive(Debug)]
pub struct Uppercase {
    pub val: char
}

#[derive(Debug)]
pub struct Lowercase {
    pub val: char
}


// not quite sure how to do a given selection of things to choose from
// need to be able to tell the sample function about the choices it has before
// the struct is constructed
// can maybe accomplish that with something like a size type and a conversion
// from an integer to the choices later
// 
// maybe an overall simpler approach is to just have a single randomness thing
// that is wide enough to capture all the interesting options and then things
// like digits, uppercase, lowercase, alphanumeric, symbols, etc is the conversion
// from the background integer distribution to the special char
// something like u128 -> (0..9) for digits
// should probably just use the total width of characters even though it is
// unlikely that all unicode is supported in passwords
pub struct Selection {
    val: char
}

pub struct PreChar {
    val: u32
}

impl Into<char> for Digit {
    fn into(self) -> char {
        self.val
    }
}

impl Into<char> for Lowercase {
    fn into(self) -> char {
        self.val
    }
}

impl Into<char> for Uppercase {
    fn into(self) -> char {
        self.val
    }
}

impl TryFrom<char> for Digit {
    type Error = String;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        if ('0'..='9').contains(&value) {
            Result::Ok(Digit { val: value })
        } else {
            Result::Err(format!("A digit must be between 0 and 9, got {value}"))
        }
    }
}

impl TryFrom<char> for Uppercase {
    type Error = String;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        if ('A'..='Z').contains(&value) {
            Result::Ok(Uppercase { val: value })
        } else {
            Result::Err(format!("An Uppercase letter must be between A and Z, got {value}"))
        }
    }
}

impl TryFrom<char> for Lowercase {
    type Error = String;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        if ('a'..='z').contains(&value) {
            Result::Ok(Lowercase { val: value })
        } else {
            Result::Err(format!("A Lowercase letter must be between a and z, got {value}"))
        }
    }
}

impl Distribution<Digit> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Digit {
        let choices = Uniform::from('0'..='9');
        let choice = choices.sample(rng);
        Digit { val: choice }
    }
}

impl Distribution<Uppercase> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Uppercase {
        let choices = Uniform::from('A'..='Z');
        let choice = choices.sample(rng);
        Uppercase { val: choice }
    }
}

impl Distribution<Lowercase> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Lowercase {
        let choices = Uniform::from('a'..='z');
        let choice = choices.sample(rng);
        Lowercase { val: choice }
    }
}

impl Distribution<PreChar> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> PreChar {
        let range = Uniform::new(0, 128);
        let mut n = range.sample(rng);
        PreChar { val: n }
    }
}

impl PreChar {
    fn restrict(self, vals: &[char]) -> char {
        let l = vals.len() as u32;
        let n = (self.val % l) as usize;
        vals[n]
   }
}

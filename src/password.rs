use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};

// specification for password requirements
// for the most part it is very likely that requirements are met just through
// randomness, but just have some enforcement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    generic: Choice<char>,
    digit: bool,
    upper: bool,
    lower: bool,
    alpha: bool,
    symbol: bool,
    length: u32, //u32 just in case someone allows for a password that is billions of characters
                 //long lol
    extra: Vec<Choice<char>>, // weird extra constraints for stupid password things
}

impl Password {
    pub fn new(generic: Choice<char>, digit: bool, upper: bool, lower: bool, alpha: bool, symbol: bool, length: u32, extra: Vec<Choice<char>>) -> Option<Self> {
        // the length of the password must be the number of 
        let mut tot = extra.len();
        if digit { tot += 1 };
        if upper { tot += 1 };
        if lower { tot += 1 };
        if alpha { tot += 1 };
        if symbol { tot += 1 };
        if (length as usize) >= tot {
            Some(Password { generic, digit, upper, lower, alpha, symbol, length, extra })
        } else {
            None
        }
    }

    pub fn standard() -> Self {
        Password { generic: generic(), digit: false, upper: false, lower: false, alpha: false, symbol: false, length: 32, extra: Vec::new() }
    }

    // really simple parsing
    // example "upper+digit" => requires uppercase and a number
    // maybe there is a use case for something more complicated like
    // digit+digit => two numbers, but for now not a concern other than for the thing that requires
    // that
    // still no good idea for the weird extra things being specified that is also
    // simple, maybe something like [1a, 7!] => requires either 1a or 7! to show up but that breaks
    // length guarantees \shrug
    pub fn from_spec(allowed: Choice<char>, length: u32, pattern: String) -> Option<Self> {
        let generic = allowed;
        let mut digit = false;
        let mut upper = false;
        let mut lower = false;
        let mut alpha = false;
        let mut symbol = false;
        for s in pattern.split("+") {
            match s {
                "digit" => digit = true,
                "upper" => upper = true,
                "lower" => lower = true,
                "alpha" => alpha = true,
                "symbol" => symbol = true,
                "" => {},
                _ => return None,
            }
        }
        Password::new(generic, digit, upper, lower, alpha, symbol, length, Vec::new())
    }

    pub fn generate(&self) -> String {
        let mut vals : Vec<u32> = (0..self.length).collect();
        vals.shuffle(&mut rand::thread_rng());

        let mut res = Vec::new();
        let digit = digit();
        let upper = upper();
        let lower = lower();
        let alpha = alpha();
        let symbol = symbol();
        // checked on generation that length is fine, can be unsafe
        if self.digit {
            let item = vals.pop().unwrap();
            res.push((item, digit.choose()));
        }

        if self.upper {
            let item = vals.pop().unwrap();
            res.push((item, upper.choose()));
        }

        if self.lower {
            let item = vals.pop().unwrap();
            res.push((item, lower.choose()));
        }

        if self.alpha {
            let item = vals.pop().unwrap();
            res.push((item, alpha.choose()));
        }

        if self.symbol {
            let item = vals.pop().unwrap();
            res.push((item, symbol.choose()));
        }

        for extra in &self.extra {
            let item = vals.pop().unwrap();
            res.push((item, extra.choose()));
        }

        for item in vals {
            res.push((item, self.generic.choose()))
        }

        res.sort_by(|a, b| a.0.cmp(&b.0));

        res.into_iter().map(|x| x.1).collect()
    }
}

// type parameter T is probably unnecessary
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice<T> {
    avail: Vec<T>,
}

impl<T> Choice<T> {
    pub fn new(avail: Vec<T>) -> Option<Self> {
        if avail.len() > 0 {
            Some(Choice { avail })
        } else {
            None
        }
    }

    pub fn choose(&self) -> &T {
        self.avail.choose(&mut rand::thread_rng()).unwrap()
    }

    fn join(&mut self, other: Choice<T>) {
        self.avail.extend(other.avail)
    }
}

pub fn upper() -> Choice<char> {
   Choice::new(('A'..='Z').collect()).unwrap()
}

pub fn lower() -> Choice<char> {
   Choice::new(('a'..='z').collect()).unwrap()
}

pub fn alpha() -> Choice<char> {
    let mut x = upper();
    x.join(lower());
    x
}

pub fn digit() -> Choice<char> {
    Choice::new(('0'..='9').collect()).unwrap()
}

pub fn alpha_num() -> Choice<char> {
    let mut x = alpha();
    x.join(digit());
    x
}

pub fn symbol() -> Choice<char> {
    Choice::new(vec![
                '~','!','@','#','$','%','^','&','*','(',')','-','_','=','+','[','{',']','}','\\','|',';',':','\'','"',',','<','.','>','/','?'
    ]).unwrap()
}

pub fn generic() -> Choice<char> {
    let mut x = alpha_num();
    x.join(symbol());
    x
}



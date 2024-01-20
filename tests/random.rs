#[cfg(test)]
mod tests {
    use turt::random::{Digit, Uppercase, Lowercase};

    #[test]
    fn random_digit() {
        let digit : Digit = rand::random();
        let digit_range: Vec<char> = ('0'..='9').collect();
        assert_eq!(digit_range.len(), 10);
        assert!(digit_range.contains(&digit.val))
    }

    #[test]
    fn random_uppercase() {
        let digit : Uppercase = rand::random();
        let uppercase_range: Vec<char>= ('A'..='Z').collect();
        assert_eq!(uppercase_range.len(), 26);
        assert!(uppercase_range.contains(&digit.val))
    }

    #[test]
    fn random_lowercase() {
        let digit : Lowercase = rand::random();
        let lowercase_range: Vec<char>= ('a'..='z').collect();
        assert_eq!(lowercase_range.len(), 26);
        assert!(lowercase_range.contains(&digit.val))
    }

}

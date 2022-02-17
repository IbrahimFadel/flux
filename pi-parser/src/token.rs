pub const SPECIAL_CHARACTERS: &str = "\n\r\t :[](){}#\",;.<>";

pub const ASSIGN: &str = "=";
pub const ARROW: &str = "->";

#[inline]
pub fn is_keyword(input: &str) -> bool {
    input == ASSIGN
        || input == ARROW
        || input
            .chars()
            .next()
            .map(|c| char::is_digit(c, 10))
            .unwrap_or(false)
}

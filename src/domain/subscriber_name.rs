use std::fmt::Display;

use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Deserialize)]
pub struct SubscriberName(String);

impl Display for SubscriberName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SubscriberName {
    pub fn parse(s: impl AsRef<str>) -> Result<SubscriberName, String> {
        let s: &str = s.as_ref();
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        static FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| FORBIDDEN_CHARACTERS.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(String::from(s)))
        }
    }
    pub fn inner(self) -> String {
        self.0
    }

    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }
    pub fn inner_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "Œ".repeat(256);
        claims::assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "Œ".repeat(257);
        claims::assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        claims::assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_names_are_rejected() {
        let name = "".to_string();
        claims::assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            claims::assert_err!(SubscriberName::parse(name.to_string()));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        claims::assert_ok!(SubscriberName::parse(name));
    }
}

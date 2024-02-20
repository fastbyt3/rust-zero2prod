use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            dbg!("Recv name: {}", s.clone());
            return Err(format!("{s} is not a valid subscriber name"));
        } else {
            return Ok(Self(s));
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_grapheme_is_not_valid() {
        let name = "ё".repeat(300);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_not_valid() {
        let name = String::from("   ");
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_with_invalid_chars_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            assert_err!(SubscriberName::parse(name.to_string()));
        }
    }

    #[test]
    fn valid_name() {
        let name = String::from("Bobby Portis");
        assert_ok!(SubscriberName::parse(name));
    }
}

extern crate data_encoding;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;


use std::fmt;
use std::str::FromStr;

use data_encoding::DecodeError;
use data_encoding::DecodeKind;
use data_encoding::HEXLOWER_PERMISSIVE;
use rand::Rng;


/// Randomly generated (probably) unique IDs.
///
/// IDs are generated as a random sequence of 128 bits.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct RndId(String);

impl RndId {
    /// Return a new, random RndId.
    pub fn new() -> RndId {
        let mut rng = rand::thread_rng();
        let id: [u8; 16] = rng.gen();
        RndId(HEXLOWER_PERMISSIVE.encode(&id))
    }
}

impl fmt::Display for RndId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl FromStr for RndId {
    type Err = DecodeError;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match HEXLOWER_PERMISSIVE.decode_len(s.len()) {
            Ok(16) => {
                // Make sure the ID is actually valid and not just the correct length.
                let mut buf = [0; 16];
                HEXLOWER_PERMISSIVE
                    .decode_mut(s.as_bytes(), &mut buf)
                    .map_err(|e| e.error)?;
                // But still store it as a string.
                Ok(RndId(String::from(s).to_lowercase()))
            },
            Ok(_) => Err(DecodeError {
                position: 0,
                kind: DecodeKind::Length,
            }),
            Err(error) => Err(error),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::RndId;

    #[test]
    fn ids_differ() {
        let id1 = RndId::new();
        let id2 = RndId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn from_string() {
        let raw_id = "ce84c2f150f72f1499d28b50c550c4c0";
        let id: RndId = raw_id.parse().unwrap();
        assert_eq!(id.to_string(), raw_id);
    }

    #[test]
    fn from_string_upper() {
        let raw_id = "CE84c2f150f72f1499D28b50c550c4c0";
        let id: RndId = raw_id.parse().unwrap();
        assert_eq!(id.to_string(), raw_id.to_lowercase());
    }

    #[test]
    #[should_panic(expected = "kind: Length")]
    fn from_string_invalid_length() {
        let raw_id = "ABC";
        let _id: RndId = raw_id.parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "DecodeError")]
    fn from_string_not_hex() {
        let raw_id = "%^84c2f150f72f1499d28b50c550c4c0";
        let _id: RndId = raw_id.parse().unwrap();
    }
}

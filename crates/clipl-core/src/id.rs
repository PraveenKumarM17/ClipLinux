//! Typed identifiers.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Allocate a new random identifier.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wrap an existing UUID.
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Return the inner UUID.
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|err| Error::invalid(format!("{}: {err}", stringify!($name))))
            }
        }
    };
}

typed_id!(
    /// Identity of a clipboard history entry.
    ClipboardItemId
);
typed_id!(
    /// Identity of a media item (GIF, sticker, clip).
    MediaItemId
);
typed_id!(
    /// Identity of a sticker pack.
    StickerPackId
);
typed_id!(
    /// Identity of a user-defined snippet.
    SnippetId
);
typed_id!(
    /// Identity of a catalogued emoji record.
    EmojiId
);
typed_id!(
    /// Identity of a privacy rule.
    PrivacyRuleId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_as_strings() {
        let id = ClipboardItemId::new();
        let parsed: ClipboardItemId = id.to_string().parse().expect("parse");
        assert_eq!(id, parsed);
    }
}

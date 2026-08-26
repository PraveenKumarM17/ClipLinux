//! SHA-256 content hashing for clipboard deduplication.

use clipl_core::ClipboardContent;
use sha2::{Digest, Sha256};

/// Hex-encoded SHA-256 of [`ClipboardContent::canonical_bytes`].
pub fn content_hash(content: &ClipboardContent) -> String {
    let digest = Sha256::digest(content.canonical_bytes());
    hex_lower(&digest)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_core::ClipboardItem;

    #[test]
    fn same_text_same_hash() {
        let a = ClipboardItem::text("hello").content;
        let b = ClipboardItem::text("hello").content;
        assert_eq!(content_hash(&a), content_hash(&b));
        assert_eq!(content_hash(&a).len(), 64);
    }

    #[test]
    fn different_text_different_hash() {
        let a = ClipboardItem::text("hello").content;
        let b = ClipboardItem::text("hello!").content;
        assert_ne!(content_hash(&a), content_hash(&b));
    }
}

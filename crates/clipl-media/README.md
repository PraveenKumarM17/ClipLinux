# clipl-media

Status: **Planned / Not implemented** (registry scaffolding only)

This crate is the extension point for GIF/sticker providers. It is used by
workspace tests. The default registry contains an offline provider that
returns no results. `LocalStickerLibrary` wraps the empty placeholder.

Do not remove this crate to “simplify” the tree. Do not claim GIFs or stickers
work.

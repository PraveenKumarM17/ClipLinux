# ClipLinux folder structure

Generated from the repository tree. Omitted as generated or vendor output:

- `.git/`
- `target/` (Rust/Tauri build + `target/release/bundle/`)
- `apps/desktop/node_modules/`
- `apps/desktop/dist/`
- `apps/desktop/.svelte-kit/`

```
ClipLinux/
├── apps
│   ├── cli
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── main.rs
│   ├── daemon
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── activation.rs
│   │       ├── lib.rs
│   │       ├── main.rs
│   │       └── picker.rs
│   └── desktop
│       ├── index.html
│       ├── package.json
│       ├── package-lock.json
│       ├── scripts
│       │   ├── prepare-bundle.sh
│       │   └── tauri.sh
│       ├── src
│       │   ├── app.css
│       │   ├── App.svelte
│       │   ├── lib
│       │   │   ├── api
│       │   │   │   └── desktop.ts
│       │   │   ├── components
│       │   │   │   ├── CategoryRail.svelte
│       │   │   │   ├── ConfirmDialog.svelte
│       │   │   │   ├── EmojiPane.svelte
│       │   │   │   ├── EmptyState.svelte
│       │   │   │   ├── HistoryItem.svelte
│       │   │   │   ├── HistoryList.svelte
│       │   │   │   ├── PickerGrid.svelte
│       │   │   │   ├── PlaceholderPane.svelte
│       │   │   │   ├── SearchBar.svelte
│       │   │   │   ├── StatusIndicator.svelte
│       │   │   │   ├── SymbolsPane.svelte
│       │   │   │   ├── TabBar.svelte
│       │   │   │   └── UniversalSearch.svelte
│       │   │   ├── stores
│       │   │   │   ├── picker.svelte.ts
│       │   │   │   ├── search.svelte.ts
│       │   │   │   └── session.svelte.ts
│       │   │   ├── types
│       │   │   └── utils
│       │   │       ├── debounce.test.ts
│       │   │       ├── debounce.ts
│       │   │       ├── escape.ts
│       │   │       ├── grid.test.ts
│       │   │       ├── grid.ts
│       │   │       ├── historyView.test.ts
│       │   │       ├── historyView.ts
│       │   │       ├── keyboard.test.ts
│       │   │       ├── keyboard.ts
│       │   │       ├── searchHits.test.ts
│       │   │       ├── searchHits.ts
│       │   │       ├── time.test.ts
│       │   │       └── time.ts
│       │   ├── main.ts
│       │   └── vite-env.d.ts
│       ├── src-tauri
│       │   ├── binaries
│       │   ├── build.rs
│       │   ├── capabilities
│       │   │   └── default.json
│       │   ├── Cargo.toml
│       │   ├── gen
│       │   │   └── schemas
│       │   │       ├── acl-manifests.json
│       │   │       ├── capabilities.json
│       │   │       ├── desktop-schema.json
│       │   │       └── linux-schema.json
│       │   ├── icons
│       │   │   ├── 128x128.png
│       │   │   ├── 32x32.png
│       │   │   ├── generate.py
│       │   │   ├── .gitkeep
│       │   │   └── icon.png
│       │   ├── linux
│       │   │   ├── postinst.sh
│       │   │   └── prerm.sh
│       │   ├── src
│       │   │   ├── clipboard.rs
│       │   │   ├── commands.rs
│       │   │   ├── dto.rs
│       │   │   ├── ipc.rs
│       │   │   ├── launch.rs
│       │   │   ├── lib.rs
│       │   │   ├── main.rs
│       │   │   ├── picker.rs
│       │   │   └── window.rs
│       │   └── tauri.conf.json
│       ├── svelte.config.js
│       ├── tsconfig.json
│       └── vite.config.ts
├── ARCHITECTURE.md
├── Cargo.lock
├── Cargo.toml
├── config.example.toml
├── CONTRIBUTING.md
├── crates
│   ├── clipl-clipboard
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── engine.rs
│   │       ├── hash.rs
│   │       ├── lib.rs
│   │       ├── memory.rs
│   │       ├── picker.rs
│   │       ├── sqlite.rs
│   │       └── store.rs
│   ├── clipl-core
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── activation.rs
│   │       ├── capabilities.rs
│   │       ├── clipboard.rs
│   │       ├── config.rs
│   │       ├── emoji.rs
│   │       ├── error.rs
│   │       ├── id.rs
│   │       ├── lib.rs
│   │       ├── media.rs
│   │       ├── paths.rs
│   │       ├── placeholders.rs
│   │       ├── platform.rs
│   │       ├── privacy.rs
│   │       ├── snippet.rs
│   │       ├── timestamp.rs
│   │       └── traits
│   │           ├── activation.rs
│   │           ├── clipboard.rs
│   │           ├── media.rs
│   │           ├── mod.rs
│   │           ├── platform.rs
│   │           └── storage.rs
│   ├── clipl-emoji
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── catalog.rs
│   │       ├── lib.rs
│   │       ├── search.rs
│   │       └── skin.rs
│   ├── clipl-media
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src
│   │       └── lib.rs
│   ├── clipl-platform
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── activation
│   │       │   ├── gnome.rs
│   │       │   ├── hyprland.rs
│   │       │   ├── kde.rs
│   │       │   ├── mod.rs
│   │       │   ├── null.rs
│   │       │   ├── sway.rs
│   │       │   ├── wayland.rs
│   │       │   ├── wlroots.rs
│   │       │   └── x11.rs
│   │       ├── clipboard
│   │       │   ├── gnome.rs
│   │       │   ├── mod.rs
│   │       │   ├── null.rs
│   │       │   ├── wayland.rs
│   │       │   └── x11.rs
│   │       ├── insert
│   │       │   ├── mod.rs
│   │       │   └── x11.rs
│   │       └── lib.rs
│   ├── clipl-privacy
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── detect.rs
│   │       └── lib.rs
│   ├── clipl-protocol
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── activation.rs
│   │       ├── lib.rs
│   │       ├── picker.rs
│   │       └── transport.rs
│   ├── clipl-snippets
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src
│   │       └── lib.rs
│   └── clipl-symbols
│       ├── Cargo.toml
│       └── src
│           └── lib.rs
├── DEVELOPMENT.md
├── docs
│   ├── architecture
│   │   ├── activation.md
│   │   ├── clipboard-engine.md
│   │   ├── daemon.md
│   │   ├── desktop-daemon-boundary.md
│   │   ├── desktop.md
│   │   ├── emoji-engine.md
│   │   ├── ipc.md
│   │   ├── README.md
│   │   ├── storage.md
│   │   └── symbols-engine.md
│   ├── contributing
│   │   └── README.md
│   ├── platform-support
│   │   ├── gnome.md
│   │   ├── README.md
│   │   └── x11.md
│   └── privacy
│       └── README.md
├── .editorconfig
├── extensions
│   ├── gnome
│   │   ├── extension.js
│   │   ├── metadata.json
│   │   ├── prefs.js
│   │   ├── README.md
│   │   └── schemas
│   │       ├── gschemas.compiled
│   │       └── org.gnome.shell.extensions.clipl.gschema.xml
│   └── kde
│       ├── clipl.desktop
│       └── README.md
├── .github
│   └── workflows
│       └── release.yml
├── .gitignore
├── LICENSE
├── LICENSE-APACHE
├── LICENSE-MIT
├── MASTER_PLAN.md
├── packages
│   ├── emoji-data
│   │   ├── aliases.json
│   │   ├── emoji.compact.json
│   │   ├── README.md
│   │   ├── scripts
│   │   │   └── generate.py
│   │   └── vendor
│   │       ├── cldr-annotations-derived-en.json
│   │       ├── cldr-annotations-en.json
│   │       └── emoji-test.txt
│   ├── sticker-packs
│   │   ├── manifest.json
│   │   └── README.md
│   ├── symbols-data
│   │   ├── kaomoji.json
│   │   ├── README.md
│   │   └── symbols.json
│   └── themes
│       ├── default-dark.json
│       └── README.md
├── packaging
│   └── linux
│       ├── install-gnome-shortcut.sh
│       ├── io.clipl.ClipLinux-autostart.desktop
│       ├── io.clipl.ClipLinux-daemon.desktop
│       ├── io.clipl.ClipLinux.desktop
│       └── README.md
├── PLATFORM_CAPABILITIES.md
├── PRIVACY_MODEL.md
├── README.md
├── ROADMAP.md
├── rustfmt.toml
├── rust-toolchain.toml
├── scripts
│   ├── check-md-links.py
│   └── check.sh
├── tasks
│   ├── 000-foundation.md
│   ├── 001-clipboard-monitoring.md
│   └── README.md
└── tests
    ├── Cargo.toml
    └── src
        └── lib.rs
```

# Platform support notes

Canonical document: [/PLATFORM_CAPABILITIES.md](../../PLATFORM_CAPABILITIES.md)

Adapters are selected by identity, then must **probe**. The foundation only
implements `linux-generic`, which reports `Unknown` for compositor features.

<script lang="ts">
  import type { UiConnection } from "../stores/session.svelte";

  let { connection }: { connection: UiConnection } = $props();

  type Chip = { label: string; tone: "ok" | "warn" | "bad" | "muted"; detail: string };

  function chip(view: UiConnection): Chip {
    if (view.kind === "starting") {
      return { label: "Starting", tone: "muted", detail: "Connecting to clipl-daemon" };
    }
    if (view.kind === "disconnected") {
      return { label: "Disconnected", tone: "bad", detail: view.message };
    }
    if (view.kind === "error") {
      return { label: "Error", tone: "bad", detail: view.message };
    }
    if (view.monitoring === "Unsupported") {
      return {
        label: "Monitoring unavailable",
        tone: "warn",
        detail: view.reason || "Clipboard monitoring is not available in this session",
      };
    }
    if (view.monitoring === "Partial") {
      return {
        label: view.reason.includes("ClipLinux extension")
          ? "Extension needed"
          : "Partial monitoring",
        tone: "warn",
        detail: view.reason || "Clipboard monitoring is limited in this session",
      };
    }
    return { label: "Connected", tone: "ok", detail: `Daemon ${view.version}` };
  }

  const view = $derived(chip(connection));
</script>

<div class="status" data-tone={view.tone} title={view.detail}>
  <span class="dot" aria-hidden="true"></span>
  <span class="label">{view.label}</span>
</div>

<style>
  .status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--chip-bg);
    color: var(--fg);
    max-width: 14rem;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--muted);
  }

  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status[data-tone="ok"] .dot {
    background: var(--ok);
  }

  .status[data-tone="warn"] .dot {
    background: var(--warn);
  }

  .status[data-tone="bad"] .dot {
    background: var(--danger);
  }
</style>

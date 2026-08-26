<script lang="ts">
  import type { TabId } from "../stores/session.svelte";

  let { active, onSelect }: { active: TabId; onSelect: (tab: TabId) => void } = $props();

  const tabs: { id: TabId; label: string }[] = [
    { id: "history", label: "History" },
    { id: "emoji", label: "Emoji" },
    { id: "symbols", label: "Symbols" },
    { id: "snippets", label: "Snips" },
  ];
</script>

<div class="tabs" role="tablist" aria-label="ClipLinux panes">
  {#each tabs as tab}
    <button
      type="button"
      role="tab"
      aria-selected={active === tab.id}
      class:active={active === tab.id}
      onclick={() => onSelect(tab.id)}
    >
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    gap: 4px;
  }

  button {
    flex: 1;
    border: 1px solid var(--border);
    background: var(--field);
    color: var(--muted);
    border-radius: 8px;
    padding: 5px 6px;
    font-size: 0.78rem;
    cursor: pointer;
  }

  button.active {
    color: var(--fg);
    border-color: var(--accent);
    background: var(--selected);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>

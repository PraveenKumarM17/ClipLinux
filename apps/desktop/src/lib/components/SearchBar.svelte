<script lang="ts">
  import { onMount } from "svelte";

  let {
    value,
    onQuery,
  }: {
    value: string;
    onQuery: (next: string) => void;
  } = $props();

  let input: HTMLInputElement | undefined;

  onMount(() => {
    input?.focus();
  });
</script>

<label class="search">
  <span class="sr-only">Search clipboard history</span>
  <span class="icon" aria-hidden="true">⌕</span>
  <input
    id="clipl-search"
    bind:this={input}
    type="search"
    placeholder="Search clipboard history…"
    autocomplete="off"
    spellcheck="false"
    value={value}
    oninput={(event) => onQuery((event.currentTarget as HTMLInputElement).value)}
  />
</label>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--border);
    background: var(--field);
    border-radius: 8px;
    padding: 0 10px;
  }

  .icon {
    color: var(--muted);
    font-size: 1rem;
  }

  input {
    flex: 1;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    padding: 8px 0;
    outline: none;
  }

  input:focus-visible {
    outline: none;
  }

  .search:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--focus-ring);
  }
</style>

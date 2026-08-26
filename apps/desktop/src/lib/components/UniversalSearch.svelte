<script lang="ts">
  import type { HistoryRow } from "../api/desktop";
  import { groupHits, type UniversalHit } from "../utils/searchHits";
  import HistoryItem from "./HistoryItem.svelte";

  let {
    hits,
    selectedKey,
    now,
    onSelect,
    onPin,
    onDelete,
    onActivate,
  }: {
    hits: UniversalHit[];
    selectedKey: string | null;
    now: number;
    onSelect: (key: string) => void;
    onPin: (id: string, pinned: boolean) => void;
    onDelete: (id: string) => void;
    onActivate: () => void;
  } = $props();

  const groups = $derived(groupHits(hits));

  $effect(() => {
    if (!selectedKey) {
      return;
    }
    document.getElementById(`search-${selectedKey}`)?.scrollIntoView({ block: "nearest" });
  });
</script>

<div class="results" role="listbox" aria-label="Search results" tabindex="-1">
  {#each groups as group}
    <h2>{group.label}</h2>
    {#each group.hits as hit (hit.key)}
      {#if hit.source === "history"}
        {@const row = hit.row as HistoryRow}
        <div id={`search-${hit.key}`}>
          <HistoryItem
            item={row}
            {now}
            selected={hit.key === selectedKey}
            onSelect={() => onSelect(hit.key)}
            {onPin}
            {onDelete}
            {onActivate}
          />
        </div>
      {:else}
        <button
          type="button"
          id={`search-${hit.key}`}
          class="pick"
          class:selected={hit.key === selectedKey}
          role="option"
          aria-selected={hit.key === selectedKey}
          aria-label={`${group.label}: ${hit.item.name}`}
          onpointerdown={() => onSelect(hit.key)}
          ondblclick={() => onActivate()}
        >
          <span class="glyph">{hit.item.glyph}</span>
          <span class="name">{hit.item.name}</span>
          {#if hit.item.favorite}
            <span class="star" aria-hidden="true">★</span>
          {/if}
        </button>
      {/if}
    {/each}
  {/each}
</div>

<style>
  .results {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: auto;
    flex: 1;
    min-height: 0;
  }

  h2 {
    margin: 10px 0 4px;
    font-size: 0.72rem;
    font-weight: 650;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h2:first-child {
    margin-top: 0;
  }

  .pick {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    border: 1px solid transparent;
    background: transparent;
    color: inherit;
    border-radius: 8px;
    padding: 8px 10px;
    cursor: pointer;
    text-align: left;
  }

  .pick.selected {
    background: var(--selected);
    border-color: var(--accent);
  }

  .glyph {
    font-size: 1.2rem;
    line-height: 1;
    min-width: 1.6rem;
    text-align: center;
  }

  .name {
    flex: 1;
    min-width: 0;
    font-size: 0.88rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .star {
    color: var(--accent);
    font-size: 0.75rem;
  }
</style>

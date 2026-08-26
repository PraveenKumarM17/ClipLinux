<script lang="ts">
  import type { HistoryRow } from "../api/desktop";
  import HistoryItem from "./HistoryItem.svelte";

  let {
    items,
    selectedId,
    now,
    onSelect,
    onPin,
    onDelete,
    onActivate,
  }: {
    items: HistoryRow[];
    selectedId: string | null;
    now: number;
    onSelect: (id: string) => void;
    onPin: (id: string, pinned: boolean) => void;
    onDelete: (id: string) => void;
    onActivate: () => void;
  } = $props();

  $effect(() => {
    if (!selectedId) {
      return;
    }
    const node = document.getElementById(`item-${selectedId}`);
    node?.scrollIntoView({ block: "nearest" });
  });
</script>

<div class="list" role="listbox" aria-label="Clipboard history" tabindex="-1">
  {#each items as item (item.id)}
    <HistoryItem
      {item}
      {now}
      selected={item.id === selectedId}
      {onSelect}
      {onPin}
      {onDelete}
      {onActivate}
    />
  {/each}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: auto;
    flex: 1;
    min-height: 0;
    padding-right: 2px;
  }
</style>

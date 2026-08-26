<script lang="ts">
  import type { PickerItem } from "../api/desktop";

  let {
    items,
    selected,
    columns,
    onSelect,
    onActivate,
  }: {
    items: PickerItem[];
    selected: number;
    columns: number;
    onSelect: (index: number) => void;
    onActivate: () => void;
  } = $props();

  $effect(() => {
    const item = items[selected];
    if (!item) {
      return;
    }
    document.getElementById(`pick-${selected}`)?.scrollIntoView({ block: "nearest" });
  });
</script>

<div
  class="grid"
  style={`--cols: ${columns}`}
  role="listbox"
  aria-label="Picker results"
  tabindex="-1"
>
  {#each items as item, index (item.base + item.glyph)}
    <button
      type="button"
      id={`pick-${index}`}
      class="cell"
      class:selected={index === selected}
      class:wide={columns === 1}
      role="option"
      aria-selected={index === selected}
      aria-label={`${item.name}${item.favorite ? ", favorite" : ""}`}
      title={item.name}
      onpointerdown={() => onSelect(index)}
      ondblclick={() => onActivate()}
    >
      <span class="glyph">{item.glyph}</span>
      {#if columns === 1}
        <span class="name">{item.name}</span>
      {/if}
      {#if item.favorite}
        <span class="star" aria-hidden="true">★</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: 4px;
    overflow: auto;
    flex: 1;
    min-height: 0;
    align-content: start;
  }

  .cell {
    position: relative;
    border: 1px solid transparent;
    background: var(--field);
    color: inherit;
    border-radius: 8px;
    min-height: 2.4rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 4px;
  }

  .cell.wide {
    justify-content: flex-start;
    padding: 6px 8px;
  }

  .cell.selected {
    border-color: var(--accent);
    background: var(--selected);
  }

  .glyph {
    font-size: 1.25rem;
    line-height: 1;
  }

  .name {
    font-size: 0.78rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .star {
    position: absolute;
    top: 2px;
    right: 3px;
    font-size: 0.6rem;
    color: var(--accent);
  }
</style>

<script lang="ts">
  import { KAOMOJI_CATEGORIES, SYMBOL_CATEGORIES } from "../api/desktop";
  import {
    copyPickerSelected,
    gridColumns,
    loadPicker,
    picker,
    selectedPicker,
  } from "../stores/picker.svelte";
  import CategoryRail from "./CategoryRail.svelte";
  import EmptyState from "./EmptyState.svelte";
  import PickerGrid from "./PickerGrid.svelte";

  const item = $derived(selectedPicker());
  const categories = $derived(picker.symbolsMode === "kaomoji" ? KAOMOJI_CATEGORIES : SYMBOL_CATEGORIES);
  const active = $derived(
    picker.symbolsMode === "kaomoji" ? picker.kaomojiCategory : picker.symbolCategory,
  );

  async function chooseCategory(category: string): Promise<void> {
    if (picker.symbolsMode === "kaomoji") {
      picker.kaomojiCategory = category;
    } else {
      picker.symbolCategory = category;
    }
    await loadPicker();
  }

  async function setMode(mode: "symbols" | "kaomoji"): Promise<void> {
    picker.symbolsMode = mode;
    await loadPicker();
  }
</script>

<div class="symbols">
  <div class="modes" role="tablist" aria-label="Symbol catalogs">
    <button type="button" class:active={picker.symbolsMode === "symbols"} onclick={() => void setMode("symbols")}>
      Symbols
    </button>
    <button type="button" class:active={picker.symbolsMode === "kaomoji"} onclick={() => void setMode("kaomoji")}>
      Kaomoji
    </button>
  </div>
  <CategoryRail {categories} {active} onSelect={(category) => void chooseCategory(category)} />
  <p class="preview" aria-live="polite">{item?.name ?? "Search symbols or kaomoji"}</p>
  {#if picker.error}
    <EmptyState title="Could not load symbols" detail={picker.error} />
  {:else if picker.items.length === 0}
    <EmptyState title="No results" detail="Nothing in this category matched." />
  {:else}
    <PickerGrid
      items={picker.items}
      selected={picker.selected}
      columns={gridColumns()}
      onSelect={(index) => (picker.selected = index)}
      onActivate={() => void copyPickerSelected()}
    />
  {/if}
</div>

<style>
  .symbols {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .modes {
    display: flex;
    gap: 4px;
  }

  .modes button {
    flex: 1;
    border: 1px solid var(--border);
    background: var(--field);
    color: var(--muted);
    border-radius: 8px;
    padding: 4px;
    cursor: pointer;
  }

  .modes button.active {
    color: var(--fg);
    border-color: var(--accent);
    background: var(--selected);
  }

  .preview {
    margin: 0;
    font-size: 0.75rem;
    color: var(--muted);
  }
</style>

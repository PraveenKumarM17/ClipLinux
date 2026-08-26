<script lang="ts">
  import { EMOJI_CATEGORIES } from "../api/desktop";
  import {
    copyPickerSelected,
    loadPicker,
    picker,
    selectedPicker,
    setSkin,
    SKIN_TONES,
    variantGlyphs,
  } from "../stores/picker.svelte";
  import CategoryRail from "./CategoryRail.svelte";
  import EmptyState from "./EmptyState.svelte";
  import PickerGrid from "./PickerGrid.svelte";

  const item = $derived(selectedPicker());
  const preview = $derived(item?.name ?? "Search or pick an emoji");

  async function chooseCategory(category: string): Promise<void> {
    picker.emojiCategory = category;
    await loadPicker();
  }
</script>

<div class="emoji">
  <div class="toolbar">
    <CategoryRail
      categories={EMOJI_CATEGORIES}
      active={picker.emojiCategory}
      onSelect={(category) => void chooseCategory(category)}
    />
    <div class="tones" role="group" aria-label="Default skin tone">
      {#each SKIN_TONES as tone}
        <button
          type="button"
          class:active={picker.skin === tone.id}
          aria-label={`Skin tone ${tone.id}`}
          aria-pressed={picker.skin === tone.id}
          onclick={() => void setSkin(tone.id)}
        >
          {tone.label}
        </button>
      {/each}
    </div>
  </div>

  <p class="preview" aria-live="polite">{preview}</p>

  {#if picker.error}
    <EmptyState title="Could not load emoji" detail={picker.error} />
  {:else if picker.items.length === 0}
    <EmptyState
      title={picker.emojiCategory === "Frequently Used" ? "No frequent emoji yet" : "No results"}
      detail={picker.emojiCategory === "Frequently Used"
        ? "Copy an emoji to build this list. It stays on this device."
        : "Nothing matched this search."}
    />
  {:else}
    <PickerGrid
      items={picker.items}
      selected={picker.selected}
      columns={8}
      onSelect={(index) => (picker.selected = index)}
      onActivate={() => void copyPickerSelected()}
    />
  {/if}

  {#if picker.variantsOpen && item}
    <div class="variants" role="dialog" aria-label="Skin tone variants">
      {#each variantGlyphs(item) as glyph}
        <button type="button" onclick={() => void copyPickerSelected(glyph)}>{glyph}</button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .emoji {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    position: relative;
  }

  .toolbar {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .tones {
    display: flex;
    gap: 4px;
  }

  .tones button {
    border: 1px solid var(--border);
    background: var(--field);
    border-radius: 6px;
    min-width: 1.8rem;
    padding: 2px 4px;
    cursor: pointer;
    color: inherit;
    font-size: 0.85rem;
  }

  .tones button.active {
    border-color: var(--accent);
    background: var(--selected);
  }

  .preview {
    margin: 0;
    font-size: 0.75rem;
    color: var(--muted);
    min-height: 1.1rem;
  }

  .variants {
    position: absolute;
    bottom: 8px;
    left: 8px;
    right: 8px;
    display: flex;
    gap: 6px;
    justify-content: center;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 8px;
    z-index: 3;
  }

  .variants button {
    border: 1px solid var(--border);
    background: var(--field);
    border-radius: 8px;
    font-size: 1.3rem;
    padding: 4px 8px;
    cursor: pointer;
  }
</style>

<script lang="ts">
  let {
    categories,
    active,
    onSelect,
  }: {
    categories: readonly string[];
    active: string;
    onSelect: (category: string) => void;
  } = $props();
</script>

<div class="rail" role="tablist" aria-label="Categories">
  {#each categories as category}
    <button
      type="button"
      role="tab"
      aria-selected={active === category}
      class:active={active === category}
      onclick={() => onSelect(category)}
    >
      {category}
    </button>
  {/each}
</div>

<style>
  .rail {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  button {
    border: 1px solid var(--border);
    background: var(--field);
    color: var(--muted);
    border-radius: 999px;
    padding: 3px 8px;
    font-size: 0.68rem;
    cursor: pointer;
  }

  button.active {
    color: var(--fg);
    border-color: var(--accent);
    background: var(--selected);
  }
</style>

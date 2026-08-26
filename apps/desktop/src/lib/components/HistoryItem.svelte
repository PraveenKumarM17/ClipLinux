<script lang="ts">
  import type { HistoryRow } from "../api/desktop";
  import { relativeTime } from "../utils/time";

  let {
    item,
    selected,
    now,
    onSelect,
    onPin,
    onDelete,
    onActivate,
  }: {
    item: HistoryRow;
    selected: boolean;
    now: number;
    onSelect: (id: string) => void;
    onPin: (id: string, pinned: boolean) => void;
    onDelete: (id: string) => void;
    onActivate: () => void;
  } = $props();
</script>

<div
  class="row"
  class:selected
  class:pinned={item.pinned}
  class:hidden={item.hidden}
  role="option"
  tabindex="-1"
  aria-selected={selected}
  id={`item-${item.id}`}
  onpointerdown={() => onSelect(item.id)}
  ondblclick={() => onActivate()}
>
  <div class="body">
    <p class="preview">{item.preview || "(empty)"}</p>
    <p class="meta">
      <time datetime={new Date(item.created_at).toISOString()}>
        {relativeTime(item.created_at, now)}
      </time>
      <span class="type">{item.content_type}</span>
      {#if item.source === "clipl"}
        <span class="src">ClipLinux</span>
      {/if}
      {#if item.hidden}
        <span class="src">hidden</span>
      {/if}
    </p>
  </div>
  <div class="actions">
    <button
      type="button"
      class="icon-btn"
      class:on={item.pinned}
      aria-label={item.pinned ? "Unpin item" : "Pin item"}
      title={item.pinned ? "Unpin" : "Pin"}
      onclick={(event) => {
        event.stopPropagation();
        onPin(item.id, item.pinned);
      }}
    >
      {item.pinned ? "📌" : "📍"}
    </button>
    <button
      type="button"
      class="icon-btn"
      aria-label="Delete item"
      title="Delete"
      disabled={item.pinned}
      onclick={(event) => {
        event.stopPropagation();
        onDelete(item.id);
      }}
    >
      🗑
    </button>
  </div>
</div>

<style>
  .row {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 8px 10px;
    border-radius: 8px;
    cursor: default;
    border: 1px solid transparent;
  }

  .row.selected {
    background: var(--selected);
    border-color: var(--accent);
  }

  .row.pinned:not(.selected) {
    background: var(--pinned-bg);
  }

  .preview {
    margin: 0;
    font-size: 0.88rem;
    line-height: 1.35;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    word-break: break-word;
  }

  .hidden .preview {
    font-style: italic;
    color: var(--muted);
  }

  .meta {
    margin: 4px 0 0;
    font-size: 0.72rem;
    color: var(--muted);
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon-btn {
    border: 0;
    background: transparent;
    cursor: pointer;
    font-size: 0.95rem;
    padding: 2px 4px;
    border-radius: 6px;
    opacity: 0.7;
  }

  .icon-btn:hover,
  .icon-btn:focus-visible {
    opacity: 1;
    background: var(--field);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .icon-btn.on {
    opacity: 1;
  }
</style>

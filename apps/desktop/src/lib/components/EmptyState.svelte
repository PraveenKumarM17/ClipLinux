<script lang="ts">
  let {
    title,
    detail,
    actionLabel,
    onAction,
    secondary,
  }: {
    title: string;
    detail: string;
    actionLabel?: string;
    onAction?: () => void;
    secondary?: string;
  } = $props();
</script>

<div class="empty" role="status">
  <p class="title">{title}</p>
  <p class="detail">{detail}</p>
  {#if secondary}
    <pre class="cmd">{secondary}</pre>
  {/if}
  {#if actionLabel && onAction}
    <button type="button" onclick={onAction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    gap: 8px;
    padding: 24px 12px;
    color: var(--muted);
  }

  .title {
    margin: 0;
    color: var(--fg);
    font-weight: 650;
  }

  .detail {
    margin: 0;
    max-width: 28ch;
    line-height: 1.4;
  }

  .cmd {
    margin: 0;
    font-size: 0.78rem;
    background: var(--field);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
    color: var(--fg);
  }

  button {
    margin-top: 4px;
    border: 1px solid var(--border);
    background: var(--field);
    color: inherit;
    border-radius: 8px;
    padding: 6px 12px;
    cursor: pointer;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>

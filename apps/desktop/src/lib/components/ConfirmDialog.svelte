<script lang="ts">
  let {
    title,
    detail,
    confirmLabel = "Confirm",
    onConfirm,
    onCancel,
  }: {
    title: string;
    detail: string;
    confirmLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="backdrop" role="presentation">
  <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title">
    <h2 id="confirm-title">{title}</h2>
    <p>{detail}</p>
    <div class="row">
      <button type="button" class="ghost" onclick={onCancel}>Cancel</button>
      <button type="button" class="danger" onclick={onConfirm}>{confirmLabel}</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: absolute;
    inset: 0;
    background: color-mix(in srgb, var(--bg) 35%, #000);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    z-index: 4;
  }

  .dialog {
    background: var(--panel);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 16px;
    max-width: 320px;
    width: 100%;
    box-shadow: 0 8px 24px rgb(0 0 0 / 18%);
  }

  h2 {
    margin: 0 0 8px;
    font-size: 1rem;
  }

  p {
    margin: 0 0 14px;
    color: var(--muted);
    font-size: 0.88rem;
    line-height: 1.4;
  }

  .row {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  button {
    border-radius: 8px;
    padding: 6px 10px;
    cursor: pointer;
    font: inherit;
  }

  .ghost {
    border: 1px solid var(--border);
    background: transparent;
    color: inherit;
  }

  .danger {
    border: 1px solid var(--danger);
    background: var(--danger);
    color: #fff;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>

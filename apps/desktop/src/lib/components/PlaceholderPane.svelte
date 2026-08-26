<script lang="ts">
  let { title, summary }: { title: string; summary: string } = $props();
</script>

<section class="pane" aria-live="polite">
  <h2>{title}</h2>
  <p>{summary}</p>
  <p class="soon">Coming next — this pane is not available yet.</p>
</section>

<style>
  .pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 24px 8px;
    text-align: center;
    color: var(--muted);
  }

  h2 {
    margin: 0 0 8px;
    color: var(--fg);
    font-size: 1.05rem;
  }

  p {
    margin: 0 0 8px;
    line-height: 1.4;
  }

  .soon {
    font-size: 0.82rem;
  }
</style>

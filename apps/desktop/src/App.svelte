<script lang="ts">
  const tabs = ["Clipboard", "Emoji", "GIFs", "Stickers", "Symbols", "Snippets"] as const;
  let active = $state<(typeof tabs)[number]>("Emoji");
</script>

<main class="shell">
  <header>
    <h1>ClipLinux</h1>
    <p>Foundation shell. Clipboard monitoring is not implemented yet.</p>
  </header>

  <nav>
    {#each tabs as tab}
      <button class:active={active === tab} onclick={() => (active = tab)}>
        {tab}
      </button>
    {/each}
  </nav>

  <section>
    <p>Selected: <strong>{active}</strong></p>
    <p>
      This UI is a layout placeholder. Search, paste, and daemon IPC land in later
      milestones.
    </p>
  </section>
</main>

<style>
  .shell {
    width: min(420px, 100%);
    padding: 16px;
  }

  h1 {
    margin: 0 0 4px;
    font-size: 1.25rem;
  }

  p {
    margin: 0 0 12px;
    color: #a1a1aa;
  }

  nav {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }

  button {
    border: 1px solid #3f3f46;
    background: #18181b;
    color: inherit;
    border-radius: 8px;
    padding: 6px 10px;
    cursor: pointer;
  }

  button.active {
    border-color: #f59e0b;
    color: #fbbf24;
  }
</style>

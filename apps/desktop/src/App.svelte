<script lang="ts">
  import { onMount } from "svelte";
  import ConfirmDialog from "./lib/components/ConfirmDialog.svelte";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import EmojiPane from "./lib/components/EmojiPane.svelte";
  import HistoryList from "./lib/components/HistoryList.svelte";
  import PlaceholderPane from "./lib/components/PlaceholderPane.svelte";
  import SearchBar from "./lib/components/SearchBar.svelte";
  import StatusIndicator from "./lib/components/StatusIndicator.svelte";
  import SymbolsPane from "./lib/components/SymbolsPane.svelte";
  import TabBar from "./lib/components/TabBar.svelte";
  import UniversalSearch from "./lib/components/UniversalSearch.svelte";
  import * as api from "./lib/api/desktop";
  import {
    closeVariants,
    copyPickerSelected,
    loadPicker,
    loadSkinPref,
    movePicker,
    openVariants,
    picker,
    toggleFavorite,
  } from "./lib/stores/picker.svelte";
  import {
    clearSearch,
    copyUniversal,
    jumpToTab,
    loadUniversal,
    moveUniversal,
    selectedHit,
    setQuery,
    toggleUniversalFavorite,
    universal,
  } from "./lib/stores/search.svelte";
  import {
    cancelConfirm,
    closeApp,
    confirmAction,
    copySelected,
    moveSelection,
    requestClear,
    requestDelete,
    requestDeleteItem,
    retryNow,
    session,
    startSession,
    togglePin,
  } from "./lib/stores/session.svelte";
  import type { TabId } from "./lib/stores/session.svelte";
  import { escapeOutcome } from "./lib/utils/escape";
  import { listSurface } from "./lib/utils/historyView";
  import { navAction } from "./lib/utils/keyboard";
  import { isUniversalQuery } from "./lib/utils/searchHits";

  let now = $state(Date.now());
  const surface = $derived(
    listSurface({
      connectionKind: session.connection.kind,
      historyError: session.historyError,
      query: session.query,
      itemCount: session.items.length,
    }),
  );
  const startCommand = $derived(
    session.connection.kind === "disconnected" ? session.connection.start_command : "cargo run -p clipl-daemon",
  );
  const errorDetail = $derived(
    session.connection.kind === "error"
      ? session.connection.message
      : session.connection.kind === "disconnected"
        ? session.connection.message
        : (session.historyError ?? ""),
  );
  const searching = $derived(isUniversalQuery(session.query));
  const searchPlaceholder = "Search history, emoji, symbols…";

  onMount(() => {
    startSession();
    let unlisten: (() => void) | undefined;
    void api.onPickerActivated(() => {
      focusSearch();
    }).then((fn) => {
      unlisten = fn;
    });
    const tick = setInterval(() => {
      now = Date.now();
    }, 15_000);
    return () => {
      unlisten?.();
      clearInterval(tick);
    };
  });

  $effect(() => {
    if (session.connection.kind === "connected") {
      void loadSkinPref();
      if (!searching && (session.tab === "emoji" || session.tab === "symbols")) {
        void loadPicker();
      }
    }
  });

  function focusSearch(): void {
    const el = document.getElementById("clipl-search") as HTMLInputElement | null;
    el?.focus();
    el?.select();
  }

  function onQuery(value: string): void {
    setQuery(value);
  }

  function selectTab(tab: TabId): void {
    session.tab = tab;
    if (searching) {
      jumpToTab(tab);
      return;
    }
    if (tab === "emoji" || tab === "symbols") {
      void loadPicker();
    }
  }

  async function afterHistoryMutation(): Promise<void> {
    await confirmAction();
    if (searching) {
      await loadUniversal();
    }
  }

  async function pinUniversal(id: string, pinned: boolean): Promise<void> {
    await togglePin(id, pinned);
    if (searching) {
      await loadUniversal();
    }
  }

  function onKeydown(event: KeyboardEvent): void {
    if (session.confirm) {
      if (event.key === "Escape") {
        event.preventDefault();
        cancelConfirm();
      } else if (event.key === "Enter") {
        event.preventDefault();
        void afterHistoryMutation();
      }
      return;
    }

    if (picker.variantsOpen && event.key === "Escape") {
      event.preventDefault();
      closeVariants();
      return;
    }

    const action = navAction(event);
    if (!action) {
      return;
    }

    if (action === "search") {
      event.preventDefault();
      focusSearch();
      return;
    }
    if (action === "escape") {
      event.preventDefault();
      if (escapeOutcome(session.query) === "clear-search") {
        clearSearch();
        focusSearch();
      } else {
        void closeApp();
      }
      return;
    }

    if (searching) {
      if (action === "up" || action === "down") {
        event.preventDefault();
        moveUniversal(action === "down" ? 1 : -1);
        return;
      }
      if (action === "copy") {
        event.preventDefault();
        void copyUniversal();
        return;
      }
      if (action === "favorite") {
        event.preventDefault();
        void toggleUniversalFavorite();
        return;
      }
      if (action === "delete") {
        event.preventDefault();
        const hit = selectedHit();
        if (hit?.source === "history") {
          requestDeleteItem(hit.row);
        }
      }
      return;
    }

    const pickerTab = session.tab === "emoji" || session.tab === "symbols";
    if (pickerTab) {
      if (action === "up") {
        event.preventDefault();
        movePicker("up");
        return;
      }
      if (action === "down") {
        event.preventDefault();
        movePicker("down");
        return;
      }
      if (action === "left") {
        event.preventDefault();
        movePicker("left");
        return;
      }
      if (action === "right") {
        event.preventDefault();
        movePicker("right");
        return;
      }
      if (action === "copy") {
        event.preventDefault();
        void copyPickerSelected();
        return;
      }
      if (action === "favorite") {
        event.preventDefault();
        void toggleFavorite();
        return;
      }
      if (action === "variants") {
        event.preventDefault();
        openVariants();
      }
      return;
    }

    if (session.tab !== "history") {
      return;
    }
    if (action === "up" || action === "down") {
      event.preventDefault();
      moveSelection(action === "down" ? 1 : -1);
      return;
    }
    if (action === "copy") {
      event.preventDefault();
      void copySelected();
      return;
    }
    if (action === "delete") {
      event.preventDefault();
      requestDelete();
      return;
    }
    if (action === "clear") {
      event.preventDefault();
      requestClear();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main class="shell">
  <header>
    <div class="brand">
      <h1>ClipLinux</h1>
      <StatusIndicator connection={session.connection} />
    </div>
  </header>

  <SearchBar value={session.query} placeholder={searchPlaceholder} onQuery={onQuery} />

  <TabBar active={session.tab} onSelect={selectTab} />

  {#if searching}
    <section class="pane" aria-label="Search results">
      {#if session.connection.kind === "starting"}
        <EmptyState title="Starting" detail="Connecting to the ClipLinux daemon…" />
      {:else if session.connection.kind === "disconnected"}
        <EmptyState
          title="ClipLinux daemon is not running."
          detail="Start the daemon, then retry. Search needs the daemon."
          secondary={startCommand}
          actionLabel="Retry"
          onAction={retryNow}
        />
      {:else if session.connection.kind === "error"}
        <EmptyState title="Daemon error" detail={errorDetail} actionLabel="Retry" onAction={retryNow} />
      {:else if universal.error}
        <EmptyState title="Search failed" detail={universal.error} />
      {:else if universal.hits.length === 0 && universal.loading}
        <EmptyState title="Searching" detail="Looking through history, emoji, symbols, and kaomoji…" />
      {:else if universal.hits.length === 0}
        <EmptyState
          title="No results"
          detail={`Nothing in history, emoji, symbols, or kaomoji matched “${session.query.trim()}”.`}
        />
      {:else}
        <UniversalSearch
          hits={universal.hits}
          selectedKey={selectedHit()?.key ?? null}
          {now}
          onSelect={(key) => {
            const idx = universal.hits.findIndex((hit) => hit.key === key);
            if (idx >= 0) {
              universal.selected = idx;
            }
          }}
          onPin={(id, pinned) => void pinUniversal(id, pinned)}
          onDelete={(id) => {
            const hit = universal.hits.find((row) => row.source === "history" && row.row.id === id);
            if (hit?.source === "history") {
              requestDeleteItem(hit.row);
            }
          }}
          onActivate={() => void copyUniversal()}
        />
      {/if}
    </section>
  {:else if session.tab === "history"}
    <section class="pane" aria-label="Clipboard history">
      {#if surface === "starting"}
        <EmptyState title="Starting" detail="Connecting to the ClipLinux daemon…" />
      {:else if surface === "disconnected"}
        <EmptyState
          title="ClipLinux daemon is not running."
          detail="Start the daemon, then retry. History is not empty — the UI cannot reach the source of truth."
          secondary={startCommand}
          actionLabel="Retry"
          onAction={retryNow}
        />
      {:else if surface === "error"}
        <EmptyState
          title="Daemon error"
          detail={errorDetail}
          actionLabel="Retry"
          onAction={retryNow}
        />
      {:else if surface === "history-error"}
        <EmptyState
          title="Could not load history"
          detail={session.historyError ?? "The history request failed."}
          actionLabel="Retry"
          onAction={retryNow}
        />
      {:else if surface === "empty"}
        <EmptyState
          title="No clipboard history yet"
          detail="Copy text in another app and it will appear here when monitoring is available."
        />
      {:else if surface === "no-results"}
        <EmptyState title="No results" detail={`Nothing matched “${session.query}”.`} />
      {:else}
        <HistoryList
          items={session.items}
          selectedId={session.selectedId}
          {now}
          onSelect={(id) => (session.selectedId = id)}
          onPin={(id, pinned) => void togglePin(id, pinned)}
          onDelete={(id) => requestDelete(id)}
          onActivate={() => void copySelected()}
        />
      {/if}
    </section>
  {:else if session.tab === "emoji"}
    <section class="pane" aria-label="Emoji"><EmojiPane /></section>
  {:else if session.tab === "symbols"}
    <section class="pane" aria-label="Symbols"><SymbolsPane /></section>
  {:else}
    <PlaceholderPane title="Snippets" summary="Named snippets are not wired yet." />
  {/if}

  {#if session.notice}
    <p class="notice" role="status">{session.notice}</p>
  {/if}

  <footer>
    <span><kbd>↑</kbd><kbd>↓</kbd><kbd>←</kbd><kbd>→</kbd> Navigate</span>
    <span><kbd>Enter</kbd> Copy</span>
    <span><kbd>Ctrl</kbd>+<kbd>D</kbd> Favorite</span>
    <span><kbd>Esc</kbd> Close</span>
  </footer>
</main>

{#if session.confirm?.kind === "delete"}
  <ConfirmDialog
    title="Delete this item?"
    detail={session.confirm.preview || "Remove this unpinned clipboard item from history."}
    confirmLabel="Delete"
    onConfirm={() => void afterHistoryMutation()}
    onCancel={cancelConfirm}
  />
{:else if session.confirm?.kind === "clear"}
  <ConfirmDialog
    title="Clear unpinned history?"
    detail="Pinned items are kept. This cannot be undone."
    confirmLabel="Clear"
    onConfirm={() => void afterHistoryMutation()}
    onCancel={cancelConfirm}
  />
{/if}

<style>
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 12px 8px;
    position: relative;
  }

  header .brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  h1 {
    margin: 0;
    font-size: 1.05rem;
    letter-spacing: -0.02em;
  }

  .pane {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .notice {
    margin: 0;
    font-size: 0.78rem;
    color: var(--fg);
    background: var(--selected);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 6px 8px;
  }

  footer {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    font-size: 0.7rem;
    color: var(--muted);
  }

  kbd {
    font: inherit;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 4px;
    background: var(--field);
    color: var(--fg);
  }
</style>

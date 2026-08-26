import type { ConnectionView, HistoryRow } from "../api/desktop";
import * as api from "../api/desktop";
import { debounce } from "../utils/debounce";
import { nextIndex } from "../utils/keyboard";

export type TabId = "history" | "emoji" | "symbols" | "snippets";

export type ConfirmState =
  | { kind: "delete"; id: string; preview: string }
  | { kind: "clear" }
  | null;

export type UiConnection = { kind: "starting" } | ConnectionView;

export const session = $state({
  tab: "history" as TabId,
  query: "",
  items: [] as HistoryRow[],
  selectedId: null as string | null,
  connection: { kind: "starting" } as UiConnection,
  historyError: null as string | null,
  notice: null as string | null,
  confirm: null as ConfirmState,
  loadingHistory: false,
});

const SEARCH_DEBOUNCE_MS = 180;
const POLL_MS = 4000;
const BACKOFF_MAX = 16_000;

let backoffMs = 1000;
let retryTimer: ReturnType<typeof setTimeout> | undefined;
let pollTimer: ReturnType<typeof setInterval> | undefined;
let noticeTimer: ReturnType<typeof setTimeout> | undefined;
let started = false;

function preserveSelection(rows: HistoryRow[]): void {
  if (rows.length === 0) {
    session.selectedId = null;
    return;
  }
  if (session.selectedId && rows.some((row) => row.id === session.selectedId)) {
    return;
  }
  session.selectedId = rows[0].id;
}

export function setNotice(message: string | null): void {
  session.notice = message;
  if (noticeTimer !== undefined) {
    clearTimeout(noticeTimer);
    noticeTimer = undefined;
  }
  if (message) {
    noticeTimer = setTimeout(() => {
      session.notice = null;
      noticeTimer = undefined;
    }, 2500);
  }
}

async function loadHistory(): Promise<void> {
  if (session.connection.kind !== "connected") {
    return;
  }
  session.loadingHistory = true;
  try {
    const rows = session.query.trim()
      ? await api.searchHistory(session.query)
      : await api.getHistory();
    session.items = rows;
    session.historyError = null;
    preserveSelection(rows);
  } catch (err) {
    session.historyError = err instanceof Error ? err.message : String(err);
    session.items = [];
  } finally {
    session.loadingHistory = false;
  }
}

const debouncedSearch = debounce(() => {
  void loadHistory();
}, SEARCH_DEBOUNCE_MS);

export function setQuery(value: string): void {
  session.query = value;
  debouncedSearch();
}

export function clearSearch(): void {
  debouncedSearch.cancel();
  session.query = "";
  void loadHistory();
}

function stopPoll(): void {
  if (pollTimer !== undefined) {
    clearInterval(pollTimer);
    pollTimer = undefined;
  }
}

function startPoll(): void {
  if (pollTimer !== undefined) {
    return;
  }
  pollTimer = setInterval(() => {
    if (session.connection.kind === "connected" && session.tab === "history") {
      void loadHistory();
    }
  }, POLL_MS);
}

function scheduleRetry(): void {
  if (retryTimer !== undefined) {
    return;
  }
  const wait = backoffMs;
  backoffMs = Math.min(BACKOFF_MAX, backoffMs * 2);
  retryTimer = setTimeout(() => {
    retryTimer = undefined;
    void refreshStatus();
  }, wait);
}

export async function refreshStatus(): Promise<void> {
  const view = await api.getDaemonStatus();
  session.connection = view;
  if (view.kind === "connected") {
    backoffMs = 1000;
    if (retryTimer !== undefined) {
      clearTimeout(retryTimer);
      retryTimer = undefined;
    }
    await loadHistory();
    startPoll();
    return;
  }
  session.items = [];
  stopPoll();
  if (view.kind === "disconnected") {
    scheduleRetry();
  }
}

export function retryNow(): void {
  if (retryTimer !== undefined) {
    clearTimeout(retryTimer);
    retryTimer = undefined;
  }
  backoffMs = 1000;
  session.connection = { kind: "starting" };
  void refreshStatus();
}

export function startSession(): void {
  if (started) {
    return;
  }
  started = true;
  void refreshStatus();
}

export function moveSelection(delta: number): void {
  if (session.items.length === 0) {
    return;
  }
  const current = Math.max(
    0,
    session.items.findIndex((item) => item.id === session.selectedId),
  );
  const next = nextIndex(current, session.items.length, delta);
  session.selectedId = session.items[next].id;
}

export function selectedItem(): HistoryRow | undefined {
  return session.items.find((item) => item.id === session.selectedId) ?? session.items[0];
}

export async function copySelected(): Promise<void> {
  const item = selectedItem();
  if (!item) {
    setNotice("Nothing to copy.");
    return;
  }
  if (item.hidden) {
    setNotice("Hidden items cannot be copied.");
    return;
  }
  try {
    await api.copyHistoryItem(item.id);
    setNotice("Copied to clipboard.");
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export function requestDelete(id?: string): void {
  const item = id
    ? session.items.find((row) => row.id === id)
    : selectedItem();
  if (!item) {
    return;
  }
  if (item.pinned) {
    setNotice("Unpin this item before deleting it.");
    return;
  }
  session.confirm = { kind: "delete", id: item.id, preview: item.preview };
}

export function requestClear(): void {
  session.confirm = { kind: "clear" };
}

export function cancelConfirm(): void {
  session.confirm = null;
}

export async function confirmAction(): Promise<void> {
  const pending = session.confirm;
  if (!pending) {
    return;
  }
  session.confirm = null;
  try {
    if (pending.kind === "delete") {
      await api.deleteHistoryItem(pending.id);
      setNotice("Item deleted.");
    } else {
      const count = await api.clearHistory();
      setNotice(count === 0 ? "No unpinned items to clear." : `Cleared ${count} unpinned items.`);
    }
    await loadHistory();
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export async function togglePin(id: string, pinned: boolean): Promise<void> {
  try {
    if (pinned) {
      await api.unpinHistoryItem(id);
    } else {
      await api.pinHistoryItem(id);
    }
    await loadHistory();
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export async function closeApp(): Promise<void> {
  await api.closeWindow();
}

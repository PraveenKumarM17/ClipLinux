import * as api from "../api/desktop";
import { debounce } from "../utils/debounce";
import { nextIndex } from "../utils/keyboard";
import {
  EMOJI_CAP,
  firstIndexForTab,
  isUniversalQuery,
  KAOMOJI_CAP,
  mergeUniversalHits,
  SYMBOL_CAP,
  type UniversalHit,
} from "../utils/searchHits";
import { loadPicker } from "./picker.svelte";
import { copyHistoryRow, noticeFromInsert, reloadHistory, session, setNotice } from "./session.svelte";

export const universal = $state({
  hits: [] as UniversalHit[],
  selected: 0,
  loading: false,
  error: null as string | null,
});

export function selectedHit(): UniversalHit | undefined {
  return universal.hits[universal.selected];
}

export function moveUniversal(delta: number): void {
  if (universal.hits.length === 0) {
    return;
  }
  universal.selected = nextIndex(universal.selected, universal.hits.length, delta);
}

export function jumpToTab(tab: string): void {
  if (universal.hits.length === 0) {
    return;
  }
  universal.selected = firstIndexForTab(universal.hits, tab);
}

async function applyBrowse(): Promise<void> {
  universal.hits = [];
  universal.selected = 0;
  universal.error = null;
  universal.loading = false;
  if (session.tab === "history") {
    await reloadHistory();
    return;
  }
  if (session.tab === "emoji" || session.tab === "symbols") {
    await loadPicker();
  }
}

export async function loadUniversal(): Promise<void> {
  if (session.connection.kind !== "connected") {
    universal.hits = [];
    universal.loading = false;
    return;
  }
  const query = session.query.trim();
  if (!query) {
    universal.hits = [];
    universal.loading = false;
    return;
  }
  universal.loading = true;
  const previous = selectedHit()?.key;
  try {
    const [history, emoji, symbols, kaomoji] = await Promise.all([
      api.searchHistory(query).catch(() => []),
      api.searchEmoji(query, EMOJI_CAP).catch(() => []),
      api.searchPicker("symbol", query, SYMBOL_CAP).catch(() => []),
      api.searchPicker("kaomoji", query, KAOMOJI_CAP).catch(() => []),
    ]);
    const hits = mergeUniversalHits({ history, emoji, symbols, kaomoji });
    universal.hits = hits;
    universal.error = null;
    const idx = previous ? hits.findIndex((hit) => hit.key === previous) : 0;
    universal.selected = idx >= 0 ? idx : 0;
  } catch (err) {
    universal.error = err instanceof Error ? err.message : String(err);
    universal.hits = [];
  } finally {
    universal.loading = false;
  }
}

async function applyQuery(): Promise<void> {
  if (isUniversalQuery(session.query)) {
    await loadUniversal();
    return;
  }
  await applyBrowse();
}

const debouncedApply = debounce(() => {
  void applyQuery();
}, 180);

export function setQuery(value: string): void {
  session.query = value;
  if (isUniversalQuery(value) && universal.hits.length === 0) {
    universal.loading = true;
  }
  debouncedApply();
}

export function clearSearch(): void {
  debouncedApply.cancel();
  session.query = "";
  void applyBrowse();
}

export async function copyUniversal(): Promise<void> {
  const hit = selectedHit();
  if (!hit) {
    setNotice("Nothing to copy.");
    return;
  }
  if (hit.source === "history") {
    await copyHistoryRow(hit.row);
    return;
  }
  try {
    const result = await api.copyPickerItem(hit.source, hit.item.glyph, hit.item.base);
    noticeFromInsert(result, `Copied ${hit.item.glyph}. Press Ctrl+V.`);
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export async function toggleUniversalFavorite(): Promise<void> {
  const hit = selectedHit();
  if (!hit || hit.source === "history") {
    return;
  }
  try {
    const next = await api.setPickerFavorite(hit.source, hit.item.base, !hit.item.favorite);
    universal.hits = universal.hits.map((row) => {
      if (row.source === "history" || row.key !== hit.key) {
        return row;
      }
      return { ...row, item: { ...row.item, favorite: next } };
    });
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

import type { HistoryRow, PickerItem } from "../api/desktop";

export const HISTORY_CAP = 10;
export const EMOJI_CAP = 20;
export const SYMBOL_CAP = 10;
export const KAOMOJI_CAP = 8;

export type PickerSource = "emoji" | "symbol" | "kaomoji";
export type UniversalSource = "history" | PickerSource;

export type UniversalHit =
  | { source: "history"; key: string; row: HistoryRow }
  | { source: PickerSource; key: string; item: PickerItem };

export type HitGroup = {
  source: UniversalSource;
  label: string;
  hits: UniversalHit[];
};

/** Non-empty search leaves tab browse and queries every catalog. */
export function isUniversalQuery(query: string): boolean {
  return query.trim().length > 0;
}

export function sourceLabel(source: UniversalSource): string {
  switch (source) {
    case "history":
      return "History";
    case "emoji":
      return "Emoji";
    case "symbol":
      return "Symbols";
    case "kaomoji":
      return "Kaomoji";
  }
}

export function tabSource(tab: string): UniversalSource | null {
  if (tab === "history") {
    return "history";
  }
  if (tab === "emoji") {
    return "emoji";
  }
  if (tab === "symbols") {
    return "symbol";
  }
  return null;
}

export function mergeUniversalHits(input: {
  history: HistoryRow[];
  emoji: PickerItem[];
  symbols: PickerItem[];
  kaomoji: PickerItem[];
}): UniversalHit[] {
  const hits: UniversalHit[] = [];
  for (const row of input.history.slice(0, HISTORY_CAP)) {
    hits.push({ source: "history", key: `history:${row.id}`, row });
  }
  for (const item of input.emoji.slice(0, EMOJI_CAP)) {
    hits.push({ source: "emoji", key: `emoji:${item.base}`, item });
  }
  for (const item of input.symbols.slice(0, SYMBOL_CAP)) {
    hits.push({ source: "symbol", key: `symbol:${item.base}`, item });
  }
  for (const item of input.kaomoji.slice(0, KAOMOJI_CAP)) {
    hits.push({ source: "kaomoji", key: `kaomoji:${item.base}`, item });
  }
  return hits;
}

export function groupHits(hits: UniversalHit[]): HitGroup[] {
  const order: UniversalSource[] = ["history", "emoji", "symbol", "kaomoji"];
  return order
    .map((source) => ({
      source,
      label: sourceLabel(source),
      hits: hits.filter((hit) => hit.source === source),
    }))
    .filter((group) => group.hits.length > 0);
}

export function firstIndexOfSource(hits: UniversalHit[], source: UniversalSource): number {
  return hits.findIndex((hit) => hit.source === source);
}

/** Jump to the catalog the tab represents; Symbols includes kaomoji. */
export function firstIndexForTab(hits: UniversalHit[], tab: string): number {
  if (tab === "symbols") {
    const symbols = firstIndexOfSource(hits, "symbol");
    if (symbols >= 0) {
      return symbols;
    }
    const kaomoji = firstIndexOfSource(hits, "kaomoji");
    return kaomoji >= 0 ? kaomoji : 0;
  }
  const source = tabSource(tab);
  if (!source) {
    return 0;
  }
  const index = firstIndexOfSource(hits, source);
  return index >= 0 ? index : 0;
}

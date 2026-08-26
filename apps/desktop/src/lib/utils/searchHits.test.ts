import { describe, expect, it } from "vitest";
import type { HistoryRow, PickerItem } from "../api/desktop";
import {
  EMOJI_CAP,
  firstIndexForTab,
  groupHits,
  isUniversalQuery,
  mergeUniversalHits,
  sourceLabel,
} from "./searchHits";

function history(id: string, preview: string): HistoryRow {
  return {
    id,
    preview,
    content_type: "text",
    created_at: 0,
    pinned: false,
    hidden: false,
    source: "x11",
  };
}

function picker(name: string, glyph = "x"): PickerItem {
  return {
    glyph,
    base: glyph,
    name,
    category: "General",
    has_skin_tones: false,
    variants: [],
    favorite: false,
  };
}

describe("universal search hits", () => {
  it("treats any non-whitespace query as universal", () => {
    expect(isUniversalQuery("")).toBe(false);
    expect(isUniversalQuery("   ")).toBe(false);
    expect(isUniversalQuery("heart")).toBe(true);
  });

  it("merges catalogs in history, emoji, symbols, kaomoji order", () => {
    const hits = mergeUniversalHits({
      history: [history("1", "meeting notes")],
      emoji: [picker("Red Heart", "❤️")],
      symbols: [picker("Euro Sign", "€")],
      kaomoji: [picker("Shrug", "¯\\_(ツ)_/¯")],
    });
    expect(hits.map((hit) => hit.source)).toEqual(["history", "emoji", "symbol", "kaomoji"]);
    expect(groupHits(hits).map((group) => group.label)).toEqual([
      "History",
      "Emoji",
      "Symbols",
      "Kaomoji",
    ]);
  });

  it("caps oversized catalogs", () => {
    const emoji = Array.from({ length: EMOJI_CAP + 5 }, (_, i) => picker(`e${i}`, `${i}`));
    const hits = mergeUniversalHits({
      history: [],
      emoji,
      symbols: [],
      kaomoji: [],
    });
    expect(hits).toHaveLength(EMOJI_CAP);
  });

  it("jumps to the matching catalog for a tab", () => {
    const hits = mergeUniversalHits({
      history: [history("1", "hello")],
      emoji: [picker("Grinning", "😀")],
      symbols: [],
      kaomoji: [picker("Lenny", "( ͡° ͜ʖ ͡°)")],
    });
    expect(hits[firstIndexForTab(hits, "emoji")].source).toBe("emoji");
    expect(hits[firstIndexForTab(hits, "symbols")].source).toBe("kaomoji");
    expect(sourceLabel("history")).toBe("History");
  });
});

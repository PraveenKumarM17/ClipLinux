import type { PickerItem, PickerKind } from "../api/desktop";
import * as api from "../api/desktop";
import { debounce } from "../utils/debounce";
import { moveGridIndex, type GridDir } from "../utils/grid";
import { noticeFromInsert, session, setNotice } from "./session.svelte";

export const SKIN_TONES = [
  { id: "default", label: "Default" },
  { id: "light", label: "🏻" },
  { id: "medium_light", label: "🏼" },
  { id: "medium", label: "🏽" },
  { id: "medium_dark", label: "🏾" },
  { id: "dark", label: "🏿" },
] as const;

export const picker = $state({
  emojiCategory: "Frequently Used",
  symbolCategory: "General",
  kaomojiCategory: "Happy",
  symbolsMode: "symbols" as "symbols" | "kaomoji",
  items: [] as PickerItem[],
  selected: 0,
  skin: "default",
  error: null as string | null,
  loading: false,
  variantsOpen: false,
});

const GRID_COLS = 8;
const KAOMOJI_COLS = 1;

export function gridColumns(): number {
  if (session.tab === "symbols" && picker.symbolsMode === "kaomoji") {
    return KAOMOJI_COLS;
  }
  return GRID_COLS;
}

export function currentKind(): PickerKind {
  if (session.tab === "emoji") {
    return "emoji";
  }
  return picker.symbolsMode === "kaomoji" ? "kaomoji" : "symbol";
}

function preservePickerSelection(items: PickerItem[]): void {
  const prev = picker.items[picker.selected]?.base;
  picker.items = items;
  const idx = prev ? items.findIndex((item) => item.base === prev) : 0;
  picker.selected = idx >= 0 ? idx : 0;
}

export async function loadPicker(): Promise<void> {
  if (session.connection.kind !== "connected") {
    return;
  }
  if (session.tab !== "emoji" && session.tab !== "symbols") {
    return;
  }
  picker.loading = true;
  try {
    const query = session.query.trim();
    const kind = currentKind();
    let items: PickerItem[];
    if (query) {
      items =
        kind === "emoji" ? await api.searchEmoji(query) : await api.searchPicker(kind, query);
    } else if (kind === "emoji") {
      items = await api.listEmojiCategory(picker.emojiCategory);
    } else {
      const category =
        picker.symbolsMode === "kaomoji" ? picker.kaomojiCategory : picker.symbolCategory;
      items = await api.listPickerCategory(kind, category);
    }
    picker.error = null;
    preservePickerSelection(items);
  } catch (err) {
    picker.error = err instanceof Error ? err.message : String(err);
    picker.items = [];
  } finally {
    picker.loading = false;
  }
}

const debouncedLoad = debounce(() => {
  void loadPicker();
}, 180);

export function schedulePickerLoad(): void {
  debouncedLoad();
}

export async function loadSkinPref(): Promise<void> {
  if (session.connection.kind !== "connected") {
    return;
  }
  try {
    picker.skin = await api.skinTonePref();
  } catch {
    picker.skin = "default";
  }
}

export async function setSkin(tone: string): Promise<void> {
  try {
    picker.skin = await api.setSkinTonePref(tone);
    await loadPicker();
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export function movePicker(dir: GridDir): void {
  picker.selected = moveGridIndex(
    picker.selected,
    gridColumns(),
    picker.items.length,
    dir,
  );
}

export function selectedPicker(): PickerItem | undefined {
  return picker.items[picker.selected];
}

export async function copyPickerSelected(glyphOverride?: string): Promise<void> {
  const item = selectedPicker();
  if (!item) {
    setNotice("Nothing to copy.");
    return;
  }
  const glyph = glyphOverride ?? item.glyph;
  try {
    const result = await api.copyPickerItem(currentKind(), glyph, item.base);
    noticeFromInsert(result, `Copied ${glyph}. Press Ctrl+V.`);
    picker.variantsOpen = false;
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export async function toggleFavorite(): Promise<void> {
  const item = selectedPicker();
  if (!item) {
    return;
  }
  try {
    const next = await api.setPickerFavorite(currentKind(), item.base, !item.favorite);
    picker.items = picker.items.map((row) =>
      row.base === item.base ? { ...row, favorite: next } : row,
    );
  } catch (err) {
    setNotice(err instanceof Error ? err.message : String(err));
  }
}

export function openVariants(): void {
  const item = selectedPicker();
  if (item?.has_skin_tones) {
    picker.variantsOpen = true;
  }
}

export function closeVariants(): void {
  picker.variantsOpen = false;
}

export function variantGlyphs(item: PickerItem): string[] {
  if (item.variants.length > 0) {
    return [item.base, ...item.variants];
  }
  return [item.glyph];
}

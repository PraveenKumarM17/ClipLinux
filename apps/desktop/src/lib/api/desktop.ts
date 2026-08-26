export type ConnectionView =
  | {
      kind: "connected";
      monitoring: "Supported" | "Partial" | "Unsupported";
      reason: string;
      version: string;
    }
  | { kind: "disconnected"; message: string; start_command: string }
  | { kind: "error"; message: string };

export interface HistoryRow {
  id: string;
  preview: string;
  content_type: string;
  created_at: number;
  pinned: boolean;
  hidden: boolean;
  source: string;
}

export type PickerKind = "emoji" | "symbol" | "kaomoji";

export interface PickerItem {
  glyph: string;
  base: string;
  name: string;
  category: string;
  has_skin_tones: boolean;
  variants: string[];
  favorite: boolean;
}

export const EMOJI_CATEGORIES = [
  "Frequently Used",
  "Smileys & Emotion",
  "People & Body",
  "Animals & Nature",
  "Food & Drink",
  "Travel & Places",
  "Activities",
  "Objects",
  "Symbols",
  "Flags",
] as const;

export const SYMBOL_CATEGORIES = [
  "General",
  "Arrows",
  "Math",
  "Currency",
  "Technical",
  "Greek",
  "Latin Extended",
  "Punctuation",
  "Shapes",
  "Stars",
  "Weather",
  "Units",
] as const;

export const KAOMOJI_CATEGORIES = [
  "Happy",
  "Sad",
  "Angry",
  "Shrug",
  "Table Flip",
  "Cute",
  "Surprised",
  "Actions",
  "Other",
] as const;

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export async function getDaemonStatus(): Promise<ConnectionView> {
  if (!isTauriRuntime()) {
    return {
      kind: "disconnected",
      message: "Open ClipLinux with `npm run tauri dev` (Vite alone cannot talk to the daemon).",
      start_command: "cd apps/desktop && npm run tauri dev",
    };
  }
  return invoke<ConnectionView>("cmd_get_daemon_status");
}

export async function getHistory(): Promise<HistoryRow[]> {
  return invoke<HistoryRow[]>("cmd_get_history");
}

export async function searchHistory(query: string): Promise<HistoryRow[]> {
  return invoke<HistoryRow[]>("cmd_search_history", { query });
}

export async function deleteHistoryItem(id: string): Promise<boolean> {
  return invoke<boolean>("cmd_delete_history_item", { id });
}

export async function clearHistory(): Promise<number> {
  return invoke<number>("cmd_clear_history");
}

export async function pinHistoryItem(id: string): Promise<boolean> {
  return invoke<boolean>("cmd_pin_history_item", { id });
}

export async function unpinHistoryItem(id: string): Promise<boolean> {
  return invoke<boolean>("cmd_unpin_history_item", { id });
}

export async function copyHistoryItem(id: string): Promise<void> {
  return invoke<void>("cmd_copy_history_item", { id });
}

export async function closeWindow(): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }
  return invoke<void>("cmd_close_window");
}

export async function searchEmoji(query: string, limit = 80): Promise<PickerItem[]> {
  return invoke<PickerItem[]>("cmd_search_emoji", { query, limit });
}

export async function listEmojiCategory(category: string, limit = 400): Promise<PickerItem[]> {
  return invoke<PickerItem[]>("cmd_list_emoji_category", { category, limit });
}

export async function searchPicker(
  kind: PickerKind,
  query: string,
  limit = 80,
): Promise<PickerItem[]> {
  return invoke<PickerItem[]>("cmd_search_picker", { kind, query, limit });
}

export async function listPickerCategory(kind: PickerKind, category: string): Promise<PickerItem[]> {
  return invoke<PickerItem[]>("cmd_list_picker_category", { kind, category });
}

export async function pickerFavorites(kind: PickerKind): Promise<PickerItem[]> {
  return invoke<PickerItem[]>("cmd_picker_favorites", { kind });
}

export async function setPickerFavorite(
  kind: PickerKind,
  glyph: string,
  favorite: boolean,
): Promise<boolean> {
  return invoke<boolean>("cmd_set_picker_favorite", { kind, glyph, favorite });
}

export async function skinTonePref(): Promise<string> {
  return invoke<string>("cmd_skin_tone_pref");
}

export async function setSkinTonePref(tone: string): Promise<string> {
  return invoke<string>("cmd_set_skin_tone_pref", { tone });
}

export async function copyPickerItem(kind: PickerKind, glyph: string, base: string): Promise<void> {
  return invoke<void>("cmd_copy_picker_item", { kind, glyph, base });
}

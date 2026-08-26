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

export type ListSurface =
  | "starting"
  | "disconnected"
  | "error"
  | "history-error"
  | "empty"
  | "no-results"
  | "items";

/** Choose which history pane to show. IPC failure never looks like empty history. */
export function listSurface(input: {
  connectionKind: string;
  historyError: string | null;
  query: string;
  itemCount: number;
}): ListSurface {
  if (input.connectionKind === "starting") {
    return "starting";
  }
  if (input.connectionKind === "disconnected") {
    return "disconnected";
  }
  if (input.connectionKind === "error") {
    return "error";
  }
  if (input.historyError) {
    return "history-error";
  }
  if (input.itemCount > 0) {
    return "items";
  }
  if (input.query.trim().length > 0) {
    return "no-results";
  }
  return "empty";
}

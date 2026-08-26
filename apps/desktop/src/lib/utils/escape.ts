export type EscapeOutcome = "clear-search" | "close";

/** Escape clears the search box first, then closes the window. */
export function escapeOutcome(search: string): EscapeOutcome {
  return search.length > 0 ? "clear-search" : "close";
}

export type EscapeOutcome = "clear-search" | "hide";

/** Escape clears the search box first, then hides the picker. */
export function escapeOutcome(search: string): EscapeOutcome {
  return search.length > 0 ? "clear-search" : "hide";
}

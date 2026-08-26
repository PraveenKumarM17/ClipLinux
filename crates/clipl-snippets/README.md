# clipl-snippets

Status: **Library scaffolding; UI not implemented**

In-memory snippet CRUD exists and is covered by tests. The daemon
`ListSnippets` request still returns an empty list. The desktop Snippets tab
is a placeholder pane.

Keep this crate as the snippet storage boundary. Do not claim snippets are a
shipped product feature.

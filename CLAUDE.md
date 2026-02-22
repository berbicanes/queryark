# DataForge — Database IDE

A desktop database management tool built with **Tauri 2 (Rust)** + **SvelteKit 5** + **TypeScript**, aiming to be a lightweight, fast alternative to TablePlus/DBeaver.

## Tech Stack

- **Desktop runtime**: Tauri 2 (Rust backend)
- **Frontend**: SvelteKit 5 with Svelte 5 runes, TypeScript
- **SQL Editor**: CodeMirror 6 (SQL highlighting, autocomplete, One Dark theme)
- **Database drivers**: SQLx (async Rust) — PostgreSQL + MySQL
- **State persistence**: @tauri-apps/plugin-store (JSON file)
- **Styling**: CSS variables, dark theme only, JetBrains Mono / Inter fonts

## Project Structure

```
src/                          # Frontend (SvelteKit)
├── lib/
│   ├── components/
│   │   ├── editor/           # SqlEditor (CodeMirror)
│   │   ├── grid/             # DataGrid, GridHeader, GridRow, GridCell, Pagination
│   │   ├── modals/           # ConnectionModal, ConfirmDialog
│   │   ├── sidebar/          # Sidebar, ConnectionList, SchemaTree, TreeNode
│   │   ├── structure/        # TableStructure, ColumnsView, IndexesView, ForeignKeysView
│   │   ├── tabs/             # TabBar, TabContent, QueryTab, TableTab
│   │   ├── StatusBar.svelte
│   │   └── Toolbar.svelte
│   ├── services/             # Tauri IPC wrappers (connectionService, queryService, schemaService)
│   ├── stores/               # Svelte 5 rune stores (connections, tabs, schema, ui)
│   ├── types/                # TypeScript types (connection, query, schema, tabs)
│   └── utils/                # formatters, sqlHelpers
├── routes/
│   ├── +page.svelte          # Main app layout
│   └── +layout.svelte
└── app.css                   # Global styles + CSS variables

src-tauri/                    # Backend (Rust)
├── src/
│   ├── commands/             # Tauri IPC command handlers
│   │   ├── connection.rs     # connect_db, disconnect_db, test_connection
│   │   ├── query.rs          # execute_query
│   │   └── schema.rs         # get_schemas, get_tables, get_columns, get_indexes,
│   │                         # get_foreign_keys, get_table_data, get_row_count,
│   │                         # update_cell, insert_row, delete_rows
│   ├── db/
│   │   ├── pool.rs           # PoolManager (Arc<RwLock<HashMap>>)
│   │   ├── traits.rs         # DbDriver async trait (11 methods)
│   │   ├── postgres.rs       # PostgreSQL implementation
│   │   ├── mysql.rs          # MySQL implementation
│   │   └── types.rs          # Type conversion helpers
│   ├── models/               # Serde structs (connection, query, schema)
│   ├── error.rs              # AppError enum
│   ├── lib.rs                # Tauri app builder + command registration
│   └── main.rs               # Entry point
└── Cargo.toml
```

## Commands

```bash
npm run dev              # Start Vite dev server (port 1420)
npm run build            # Build frontend
npm run tauri dev        # Run full Tauri app in dev mode
npm run tauri build      # Build production binary
npm run check            # TypeScript/Svelte type checking
```

## Current State (v0.1.0)

### What works:
- Connect to PostgreSQL and MySQL databases
- Browse schemas → tables → columns in sidebar tree
- Open table tabs (data view + structure view with columns/indexes/FK tabs)
- Open query tabs with CodeMirror SQL editor (syntax highlighting, autocomplete, Ctrl+Enter to run)
- Execute queries and view results in paginated data grid
- Inline cell editing (double-click), row insertion, row deletion
- Connection management (add/edit/delete/test) persisted to disk
- Tab system with deduplication (reopening same table activates existing tab)
- Status bar showing connection, execution time, row count
- Dark theme with CSS variables

### Known issues to fix:
- **SQL injection risk**: update_cell, insert_row, delete_rows use string concatenation instead of parameterized queries
- **Passwords stored in plaintext**: plugin-store saves JSON to disk unencrypted
- Connection pool hardcoded to 5, no idle timeout or health checks
- No query timeout enforcement
- Full result sets loaded into memory (no streaming)
- Schema cache never auto-invalidates

---

## Roadmap — Next Steps

### Phase 1: Security & Stability (Critical)
- [ ] **Parameterized queries**: Replace string concatenation in update_cell/insert_row/delete_rows with sqlx bind parameters
- [ ] **Query timeout**: Add configurable timeout to prevent runaway queries
- [ ] **Connection health checks**: Validate connections before reuse, auto-reconnect on failure
- [ ] **Error handling improvements**: Structured errors with context, query details in error messages, user-friendly messages on frontend
- [ ] **Logging**: Add log crate usage throughout Rust backend

### Phase 2: Core Data Grid Features
- [ ] **Column sorting**: Click column headers to sort ASC/DESC, multi-column sort with Shift+click
- [ ] **Column filtering**: Filter bar per column (text contains, equals, greater than, etc.)
- [ ] **Column resizing**: Drag column borders to resize
- [ ] **Column reordering**: Drag and drop columns
- [ ] **Row selection**: Checkbox column for selecting rows, Shift+click range select
- [ ] **NULL handling**: Dedicated "Set NULL" option in cell editor
- [ ] **Copy/paste**: Copy cells/rows to clipboard, paste support
- [ ] **Cell context menu**: Copy, paste, set NULL, copy as INSERT, filter by value

### Phase 3: Query Editor Enhancements
- [ ] **Schema-aware autocomplete**: Feed table/column names from schema cache into CodeMirror completions
- [ ] **Query history**: Persist executed queries, searchable history panel
- [ ] **Multi-statement execution**: Split by semicolons, execute sequentially, show multiple result sets
- [ ] **Query formatting**: Auto-format SQL (integrate sql-formatter or similar)
- [ ] **Error highlighting**: Mark the error position in the editor when a query fails
- [ ] **Saved queries**: Save/load named queries per connection
- [ ] **Comment/uncomment**: Ctrl+/ to toggle line comments

### Phase 4: Data Export & Import
- [ ] **Export to CSV**: Export current result set or table data
- [ ] **Export to JSON**: Export as JSON array
- [ ] **Export to SQL**: Generate INSERT statements
- [ ] **Export DDL**: Generate CREATE TABLE statements
- [ ] **Import CSV**: Bulk load data from CSV files
- [ ] **Copy as**: Copy selected rows as CSV/JSON/INSERT/Markdown

### Phase 5: Schema Browser Improvements
- [ ] **Views**: Show views separately from tables in the tree
- [ ] **Functions/Procedures**: List stored functions and procedures
- [ ] **Sequences**: Show sequences (PostgreSQL)
- [ ] **Enums/Types**: Show custom types (PostgreSQL)
- [ ] **Search**: Filter/search within the schema tree
- [ ] **Table stats**: Show row count and size in tree tooltips
- [ ] **Refresh**: Manual and auto-refresh for schema tree

### Phase 6: Advanced Database Operations
- [ ] **Transaction support**: BEGIN/COMMIT/ROLLBACK controls in the UI
- [ ] **Query plan visualization**: EXPLAIN ANALYZE with visual tree
- [ ] **Bulk editing**: Edit multiple cells/rows before committing
- [ ] **Undo/redo for data changes**: Track changes locally before flushing
- [ ] **Table creation**: GUI for creating new tables
- [ ] **Table alteration**: Add/modify/drop columns through UI
- [ ] **Index management**: Create/drop indexes through UI

### Phase 7: Connection & UX Polish
- [ ] **SSH tunneling**: Connect through SSH tunnel
- [ ] **SSL certificate configuration**: CA cert, client cert, client key file pickers
- [ ] **Connection URL input**: Parse and fill form from connection string
- [ ] **Keychain integration**: Store passwords in OS keychain instead of plaintext
- [ ] **Connection groups/folders**: Organize connections
- [ ] **Light theme**: Add light theme option with theme toggle
- [ ] **Keyboard shortcuts panel**: Show all shortcuts, allow customization
- [ ] **Tab drag-and-drop**: Reorder tabs by dragging
- [ ] **Split panes**: View multiple tabs side by side
- [ ] **Global search**: Ctrl+P to search tables, queries, connections

### Phase 8: Additional Database Support
- [ ] **SQLite**: Add SQLite driver (file-based)
- [ ] **MariaDB**: Verify compatibility with MySQL driver
- [ ] **MongoDB**: Add MongoDB support (different paradigm — document viewer)

## Architecture Notes

- **DbDriver trait** (`src-tauri/src/db/traits.rs`): All database-specific logic goes behind this async trait. When adding a new database, implement the 11 trait methods and register in pool.rs.
- **Stores use Svelte 5 runes** (`$state`, `$derived`): No legacy Svelte stores. All reactive state uses the runes API.
- **IPC pattern**: Frontend services in `src/lib/services/` call `invoke()` from `@tauri-apps/api/core`, which maps to `#[tauri::command]` functions in Rust.
- **Schema cache** (`src/lib/stores/schema.svelte.ts`): Per-connection cache keyed by connection ID. Lazy-loads on tree expand. Clear on disconnect.
- **CSS variables**: All colors/spacing defined in `app.css` `:root`. Components use `var(--name)` exclusively — changing theme means swapping variable values.

## Code Conventions

- Rust: Standard formatting (`cargo fmt`), thiserror for error types, async-trait for trait objects
- Frontend: TypeScript strict mode, Svelte 5 runes (`$state`, `$derived`, `$effect`), no legacy `$:` reactive statements
- Styles: Scoped `<style>` blocks in Svelte components, global variables in app.css
- Naming: camelCase for TS/Svelte, snake_case for Rust, kebab-case for CSS classes

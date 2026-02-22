# DataForge — Database IDE

A desktop database management tool built with **Tauri 2 (Rust)** + **SvelteKit 5** + **TypeScript**, aiming to be a lightweight, fast alternative to TablePlus/DBeaver.

## Tech Stack

- **Desktop runtime**: Tauri 2 (Rust backend)
- **Frontend**: SvelteKit 5 with Svelte 5 runes, TypeScript
- **SQL Editor**: CodeMirror 6 (SQL highlighting, autocomplete, One Dark theme)
- **Database drivers**: sqlx (PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift), tiberius (MSSQL), clickhouse, mongodb, scylla (Cassandra/ScyllaDB), redis, neo4rs (Neo4j), aws-sdk-dynamodb (DynamoDB)
- **State persistence**: @tauri-apps/plugin-store (JSON file)
- **Styling**: CSS variables, dark theme only, JetBrains Mono / Inter fonts

## Supported Databases (17)

| Group | Databases | Driver |
|-------|-----------|--------|
| SQL | PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift | sqlx |
| SQL | MSSQL | tiberius + bb8 |
| SQL | ClickHouse | clickhouse (HTTP) |
| SQL (stubs) | Oracle, Snowflake, BigQuery | Feature-gated, not yet implemented |
| NoSQL — Document | MongoDB, DynamoDB | mongodb, aws-sdk-dynamodb |
| NoSQL — Wide Column | Cassandra, ScyllaDB | scylla |
| NoSQL — Key-Value | Redis | redis |
| NoSQL — Graph | Neo4j | neo4rs |

## Project Structure

```
src/                          # Frontend (SvelteKit)
├── lib/
│   ├── components/
│   │   ├── editor/           # SqlEditor (CodeMirror, multi-dialect)
│   │   ├── grid/             # DataGrid, GridHeader, GridRow, GridCell, Pagination
│   │   ├── modals/           # ConnectionModal (dynamic form for 17 DBs), ConfirmDialog
│   │   ├── sidebar/          # Sidebar, ConnectionList, SchemaTree, TreeNode
│   │   ├── structure/        # TableStructure, ColumnsView, IndexesView, ForeignKeysView
│   │   ├── tabs/             # TabBar, TabContent, QueryTab, TableTab,
│   │   │                     # DocumentTab, KeyValueTab, GraphTab
│   │   ├── viewers/          # JsonViewer, KeyValueViewer
│   │   ├── StatusBar.svelte
│   │   └── Toolbar.svelte
│   ├── services/             # Tauri IPC wrappers
│   │   ├── tauri.ts          # All invoke() wrappers (35+ commands)
│   │   ├── connectionService.ts
│   │   ├── queryService.ts
│   │   ├── schemaService.ts  # SQL-specific + generic container/item/field loaders
│   │   ├── documentService.ts # MongoDB/DynamoDB CRUD
│   │   ├── keyvalueService.ts # Redis operations
│   │   └── graphService.ts   # Neo4j browsing
│   ├── stores/               # Svelte 5 rune stores (connections, tabs, schema, ui)
│   ├── types/
│   │   ├── connection.ts     # DatabaseType (17), ConnectionConfig, CloudAuth
│   │   ├── database.ts       # DB_METADATA constant, DB_GROUPS
│   │   ├── query.ts          # QueryResponse, CellValue, ColumnDef
│   │   ├── schema.ts         # SQL-specific + generic (ContainerInfo, ItemInfo, FieldInfo)
│   │   └── tabs.ts           # TabType: query | table | document | keyvalue | graph
│   └── utils/                # formatters, sqlHelpers
├── routes/
│   ├── +page.svelte          # Main app layout
│   └── +layout.svelte
└── app.css                   # Global styles + CSS variables + badge styles (17 DBs)

src-tauri/                    # Backend (Rust)
├── src/
│   ├── commands/
│   │   ├── connection.rs     # connect_db, disconnect_db, test_connection
│   │   ├── query.rs          # execute_query
│   │   ├── schema.rs         # Generic: get_containers, get_items, get_item_fields,
│   │   │                     #   get_item_data, get_item_count, get_database_category
│   │   │                     # SQL: get_schemas, get_tables, get_columns, get_indexes,
│   │   │                     #   get_foreign_keys, get_table_data, get_row_count,
│   │   │                     #   update_cell, insert_row, delete_rows
│   │   ├── document.rs       # insert_document, update_document, delete_documents
│   │   ├── keyvalue.rs       # get_value, set_value, delete_keys, get_key_type, scan_keys
│   │   └── graph.rs          # get_labels, get_relationship_types, get_node_properties, get_nodes
│   ├── db/
│   │   ├── pool.rs           # PoolManager (HashMap<String, Arc<DriverHandle>>)
│   │   ├── handle.rs         # DriverHandle enum (Sql, Document, KeyValue, Graph)
│   │   ├── traits.rs         # Trait hierarchy: DbDriver, SqlDriver, DocumentDriver,
│   │   │                     #   KeyValueDriver, GraphDriver
│   │   ├── types.rs          # Type conversion helpers (PG/MySQL)
│   │   └── drivers/
│   │       ├── postgres.rs   # PostgreSQL (sqlx)
│   │       ├── mysql.rs      # MySQL (sqlx)
│   │       ├── mariadb.rs    # MariaDB (wraps MySqlDriver)
│   │       ├── sqlite.rs     # SQLite (sqlx)
│   │       ├── cockroachdb.rs # CockroachDB (wraps PostgresDriver)
│   │       ├── redshift.rs   # Redshift (wraps PostgresDriver)
│   │       ├── mssql.rs      # MSSQL (tiberius + bb8)
│   │       ├── clickhouse.rs # ClickHouse (HTTP)
│   │       ├── mongodb.rs    # MongoDB (DocumentDriver)
│   │       ├── cassandra.rs  # Cassandra/ScyllaDB (SqlDriver via CQL)
│   │       ├── redis.rs      # Redis (KeyValueDriver)
│   │       ├── neo4j.rs      # Neo4j (GraphDriver)
│   │       ├── dynamodb.rs   # DynamoDB (DocumentDriver)
│   │       ├── oracle.rs     # Oracle (stub, feature-gated)
│   │       ├── snowflake.rs  # Snowflake (stub, feature-gated)
│   │       └── bigquery.rs   # BigQuery (stub, feature-gated)
│   ├── models/               # Serde structs (connection, query, schema)
│   ├── error.rs              # AppError enum
│   ├── lib.rs                # Tauri app builder + command registration (35+ commands)
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

## Current State (v0.2.0)

### What works:
- Connect to 14 database engines (PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift, MSSQL, ClickHouse, MongoDB, Cassandra/ScyllaDB, Redis, Neo4j, DynamoDB)
- Dynamic connection modal with grouped database selector and conditional fields per database type
- Browse schemas/containers in sidebar tree (SQL: schemas > tables > columns; NoSQL: containers > items > fields)
- Open table tabs (data view + structure view with columns/indexes/FK tabs) for SQL databases
- Open document tabs with JSON viewer for MongoDB/DynamoDB
- Open key-value tabs with type-aware viewer (string/list/set/hash/zset) for Redis
- Open graph tabs with label browser and node data grid for Neo4j
- Query tabs with CodeMirror SQL editor (multi-dialect: PostgreSQL, MySQL, SQLite, MSSQL, Cassandra)
- Execute queries and view results in paginated data grid
- Inline cell editing (double-click), row insertion, row deletion for SQL databases
- Document CRUD (insert/update/delete) for MongoDB/DynamoDB
- Key-value operations (get/set/delete/scan) for Redis
- Connection management (add/edit/delete/test) persisted to disk
- Tab system with deduplication for all tab types
- DB_METADATA-driven badges for all 17 database types in sidebar/toolbar
- Status bar showing connection, execution time, row count
- Dark theme with CSS variables

### Stub databases (feature-gated, not yet functional):
- Oracle (`cargo build --features oracle` — requires Oracle Instant Client)
- Snowflake (`cargo build --features snowflake`)
- BigQuery (`cargo build --features bigquery`)

### Known issues to fix:
- **SQL injection risk**: update_cell, insert_row, delete_rows use string concatenation instead of parameterized queries
- **Passwords stored in plaintext**: plugin-store saves JSON to disk unencrypted
- Connection pool hardcoded to 5, no idle timeout or health checks
- No query timeout enforcement
- Full result sets loaded into memory (no streaming)
- Schema cache never auto-invalidates

---

## Roadmap — Next Steps

### Phase 1: Security & Stability (Critical) ✅
- [x] **Parameterized queries**: Replace string concatenation in update_cell/insert_row/delete_rows with sqlx bind parameters
- [x] **Query timeout**: Add configurable timeout to prevent runaway queries
- [x] **Connection health checks**: Validate connections before reuse, auto-reconnect on failure
- [x] **Error handling improvements**: Structured errors with context, query details in error messages, user-friendly messages on frontend
- [x] **Logging**: Add log crate usage throughout Rust backend

### Phase 2: Core Data Grid Features ✅
- [x] **Column sorting**: Click column headers to sort ASC/DESC, multi-column sort with Shift+click
- [x] **Column filtering**: Filter bar per column (text contains, equals, greater than, etc.)
- [x] **Column resizing**: Drag column borders to resize
- [x] **Column reordering**: Drag and drop columns
- [x] **Row selection**: Checkbox column for selecting rows, Shift+click range select
- [x] **NULL handling**: Dedicated "Set NULL" option in cell editor
- [x] **Copy/paste**: Copy cells/rows to clipboard, paste support
- [x] **Cell context menu**: Copy, paste, set NULL, copy as INSERT, filter by value

### Phase 3: Query Editor Enhancements ✅
- [x] **Schema-aware autocomplete**: Feed table/column names from schema cache into CodeMirror completions
- [x] **Query history**: Persist executed queries, searchable history panel
- [x] **Multi-statement execution**: Split by semicolons, execute sequentially, show multiple result sets
- [x] **Query formatting**: Auto-format SQL (integrate sql-formatter or similar)
- [x] **Error highlighting**: Mark the error position in the editor when a query fails
- [x] **Saved queries**: Save/load named queries per connection
- [x] **Comment/uncomment**: Ctrl+/ to toggle line comments

### Phase 4: Data Export & Import
- [ ] **Export to CSV**: Export current result set or table data
- [ ] **Export to JSON**: Export as JSON array
- [ ] **Export to SQL**: Generate INSERT statements
- [ ] **Export DDL**: Generate CREATE TABLE statements
- [ ] **Import CSV**: Bulk load data from CSV files
- [ ] **Copy as**: Copy selected rows as CSV/JSON/INSERT/Markdown

### Phase 5: Schema Browser Improvements ✅
- [x] **Views**: Show views separately from tables in the tree
- [x] **Functions/Procedures**: List stored functions and procedures
- [x] **Sequences**: Show sequences (PostgreSQL)
- [x] **Enums/Types**: Show custom types (PostgreSQL)
- [x] **Search**: Filter/search within the schema tree
- [x] **Table stats**: Show row count and size in tree tooltips
- [x] **Refresh**: Manual and auto-refresh for schema tree

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

### Phase 8: Complete Stub Databases
- [ ] **Oracle**: Implement full driver using oracle crate (requires Oracle Instant Client)
- [ ] **Snowflake**: Implement full driver using snowflake-api crate (REST-based)
- [ ] **BigQuery**: Implement full driver using gcp-bigquery-client crate (REST-based)

## Architecture Notes

- **Trait hierarchy** (`src-tauri/src/db/traits.rs`):
  - `DbDriver` (base, all 14 implement): `execute_raw`, `category`, `get_containers`, `get_items`, `get_item_fields`, `get_item_data`, `get_item_count`
  - `SqlDriver: DbDriver` (relational + analytics + CQL): adds `get_schemas`, `get_tables`, `get_columns`, `get_indexes`, `get_foreign_keys`, `update_cell`, `insert_row`, `delete_rows`
  - `DocumentDriver: DbDriver` (MongoDB, DynamoDB): adds `insert_document`, `update_document`, `delete_documents`
  - `KeyValueDriver: DbDriver` (Redis): adds `get_value`, `set_value`, `delete_keys`, `get_key_type`, `scan_keys`
  - `GraphDriver: DbDriver` (Neo4j): adds `get_labels`, `get_relationship_types`, `get_node_properties`, `get_nodes`
- **DriverHandle enum** (`src-tauri/src/db/handle.rs`): Wraps `Arc<dyn SqlDriver>`, `Arc<dyn DocumentDriver>`, etc. Stored as `Arc<DriverHandle>` in PoolManager. Provides `base()`, `as_sql()`, `as_document()`, `as_keyvalue()`, `as_graph()` accessors.
- **DB_METADATA** (`src/lib/types/database.ts`): Per-database metadata constant (category, defaultPort, requiresHost, requiresFilePath, queryLanguage, badge, badgeClass, containerLabel, itemLabel, fieldLabel). Drives the dynamic connection modal, sidebar badges, and schema tree labels.
- **Schema cache** (`src/lib/stores/schema.svelte.ts`): Dual cache — `SchemaCache` for SQL-specific data (schemas/tables/columns/indexes/FKs) and `BrowserCache` for generic data (containers/items/fields). Per-connection, lazy-loads on tree expand, clears on disconnect.
- **Stores use Svelte 5 runes** (`$state`, `$derived`): No legacy Svelte stores. All reactive state uses the runes API.
- **IPC pattern**: Frontend services in `src/lib/services/` call `invoke()` from `@tauri-apps/api/core`, which maps to `#[tauri::command]` functions in Rust.
- **CSS variables**: All colors/spacing defined in `app.css` `:root`. Components use `var(--name)` exclusively — changing theme means swapping variable values.

## Code Conventions

- Rust: Standard formatting (`cargo fmt`), thiserror for error types, async-trait for trait objects
- Frontend: TypeScript strict mode, Svelte 5 runes (`$state`, `$derived`, `$effect`), no legacy `$:` reactive statements
- Styles: Scoped `<style>` blocks in Svelte components, global variables in app.css
- Naming: camelCase for TS/Svelte, snake_case for Rust, kebab-case for CSS classes

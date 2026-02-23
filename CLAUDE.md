# QueryArk — Database IDE

A desktop database management tool built with **Tauri 2 (Rust)** + **SvelteKit 5** + **TypeScript**, aiming to be a lightweight, fast alternative to TablePlus/DBeaver.

## Tech Stack

- **Desktop runtime**: Tauri 2 (Rust backend)
- **Frontend**: SvelteKit 5 with Svelte 5 runes, TypeScript
- **SQL Editor**: CodeMirror 6 (SQL highlighting, autocomplete, One Dark theme)
- **Database drivers**: sqlx (PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift), tiberius (MSSQL), clickhouse, snowflake-api (Snowflake), gcp-bigquery-client (BigQuery), mongodb, scylla (Cassandra/ScyllaDB), redis, neo4rs (Neo4j), aws-sdk-dynamodb (DynamoDB)
- **State persistence**: @tauri-apps/plugin-store (JSON file)
- **Styling**: CSS variables, dark theme only, JetBrains Mono / Inter fonts

## Supported Databases (17)

| Group | Databases | Driver |
|-------|-----------|--------|
| SQL | PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift | sqlx |
| SQL | MSSQL | tiberius + bb8 |
| SQL | ClickHouse | clickhouse (HTTP) |
| Analytics | Snowflake | snowflake-api (REST) |
| Analytics | BigQuery | gcp-bigquery-client (REST) |
| SQL (stub) | Oracle | Feature-gated, not yet implemented |
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
│   │   ├── modals/           # ConnectionModal, ConfirmDialog, CreateTableModal,
│   │   │                     # AlterTableModal, IndexModal, SettingsModal
│   │   ├── sidebar/          # Sidebar, ConnectionList, SchemaTree, TreeNode, VirtualTreeList
│   │   ├── structure/        # TableStructure, ColumnsView, IndexesView, ForeignKeysView
│   │   ├── tabs/             # TabBar, TabContent, QueryTab, TableTab,
│   │   │                     # DocumentTab, KeyValueTab, GraphTab,
│   │   │                     # DiagramTab, TableDiffTab, DataDiffTab, VisualQueryTab
│   │   ├── diagram/          # ERDiagramCanvas, DiagramToolbar, DiagramMinimap
│   │   ├── diff/             # DiffConnectionPicker, ColumnsDiff, IndexesDiff,
│   │   │                     # ForeignKeysDiff, DataDiffGrid
│   │   ├── visualquery/      # VQCanvas, VQClausePanel, VQTablePicker
│   │   ├── viewers/          # JsonViewer, KeyValueViewer
│   │   ├── StatusBar.svelte
│   │   └── Toolbar.svelte
│   ├── services/             # Tauri IPC wrappers
│   │   ├── tauri.ts          # All invoke() wrappers (35+ commands)
│   │   ├── connectionService.ts
│   │   ├── keepaliveService.ts # Connection heartbeat + auto-reconnect
│   │   ├── queryService.ts
│   │   ├── schemaService.ts  # SQL-specific + generic container/item/field loaders
│   │   ├── documentService.ts # MongoDB/DynamoDB CRUD
│   │   ├── keyvalueService.ts # Redis operations
│   │   └── graphService.ts   # Neo4j browsing
│   ├── stores/               # Svelte 5 rune stores (connections, tabs, schema, ui,
│   │                         # transaction, changeTracker)
│   ├── types/
│   │   ├── connection.ts     # DatabaseType (17), ConnectionConfig, CloudAuth
│   │   ├── database.ts       # DB_METADATA constant, DB_GROUPS
│   │   ├── query.ts          # QueryResponse, CellValue, ColumnDef
│   │   ├── schema.ts         # SQL-specific + generic (ContainerInfo, ItemInfo, FieldInfo)
│   │   ├── tabs.ts           # TabType: query | table | document | keyvalue | graph | diagram | tablediff | datadiff | visualquery
│   │   ├── diagram.ts        # DiagramTable, DiagramColumn, DiagramRelationship
│   │   ├── diff.ts           # ColumnDiff, IndexDiff, ForeignKeyDiff, TableDiffResult, DataDiffResult
│   │   └── visualQuery.ts    # VQTable, VQJoin, VQWhereClause, VQState
│   └── utils/                # formatters, sqlHelpers, diagramLayout,
│                             # schemaDiff, dataDiff, migrationGenerator, visualQueryBuilder
├── routes/
│   ├── +page.svelte          # Main app layout
│   └── +layout.svelte
└── app.css                   # Global styles + CSS variables + badge styles (17 DBs)

src-tauri/                    # Backend (Rust)
├── src/
│   ├── commands/
│   │   ├── connection.rs     # connect_db, disconnect_db, test_connection
│   │   ├── query.rs          # execute_query, cancel_query
│   │   ├── schema.rs         # Generic: get_containers, get_items, get_item_fields,
│   │   │                     #   get_item_data, get_item_count, get_database_category
│   │   │                     # SQL: get_schemas, get_tables, get_columns, get_indexes,
│   │   │                     #   get_foreign_keys, get_table_data, get_row_count,
│   │   │                     #   update_cell, insert_row, delete_rows
│   │   ├── transaction.rs    # begin_transaction, commit_transaction, rollback_transaction
│   │   ├── document.rs       # insert_document, update_document, delete_documents
│   │   ├── keyvalue.rs       # get_value, set_value, delete_keys, get_key_type, scan_keys
│   │   └── graph.rs          # get_labels, get_relationship_types, get_node_properties, get_nodes
│   ├── db/
│   │   ├── cancel.rs         # CancellationRegistry (query cancellation via oneshot channels)
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
- Connect to 16 database engines (PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift, MSSQL, ClickHouse, Snowflake, BigQuery, MongoDB, Cassandra/ScyllaDB, Redis, Neo4j, DynamoDB)
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
- Light theme (Catppuccin Latte) with theme toggle and persistent preference
- Transaction support (BEGIN/COMMIT/ROLLBACK) for PostgreSQL, MySQL, MariaDB, SQLite, CockroachDB, Redshift
- Query plan visualization (EXPLAIN ANALYZE with tree view)
- Bulk editing mode with undo/redo and visual change indicators
- Table creation GUI with live DDL preview
- Table alteration (add/drop columns) with DDL preview
- Index management (create/drop indexes) with DDL preview
- Connection groups/folders with collapsible sidebar sections
- Configurable keyboard shortcuts with shortcuts panel (Ctrl+K)
- Tab drag-and-drop reordering with visual drop indicator
- Tab context menu (Close, Close Others, Close All, Duplicate, Pin/Unpin, Split Right)
- Tab pinning — pinned tabs sorted left, close button hidden, survive Close All/Close Others
- Split panes — view two tabs side by side with draggable divider
- Command palette (Ctrl+P) — fuzzy search connections, tables, queries, actions
- Sidebar collapse/expand (Ctrl+B)
- Settings modal — editor/grid font sizes, default page size, confirm-before-delete toggle
- Connection color coding — 8-color palette, colored sidebar border, colored tab stripe
- Connection duplication — right-click → Duplicate creates copy with "(copy)" suffix
- DDL viewer — dedicated sub-tab showing CREATE TABLE DDL with copy button
- Type-aware cell editing — boolean checkbox toggle, JSON/long-text textarea, NULL pill badges
- Query cancellation — cancel running queries with Cancel button, backend tokio::select! with oneshot channels
- Window state persistence — remembers window size, position, sidebar width/collapsed, maximized state across restarts
- Session restore — reopens last active tabs and auto-reconnects on launch, configurable in settings
- Confirmation dialogs — guards tab close, close all/others, disconnect with open tabs, destructive SQL (DROP/TRUNCATE/DELETE), Redis key deletion
- Empty states & onboarding — first-launch welcome screen with CTA and DB badges, improved no-tabs empty state
- About screen — version info, license, tech stack, GitHub link, version displayed in status bar
- Connection URL parsing — paste postgres://, mysql://, mongodb://, redis://, bolt://, sqlite: URLs to auto-fill connection form
- OS keychain integration — store passwords in macOS Keychain, Windows Credential Manager, or Linux Secret Service instead of plaintext JSON; lock icon toggle on password field
- SSL certificate configuration — CA cert, client cert, client key file pickers for PostgreSQL, MySQL, MariaDB, CockroachDB, Redshift, and MongoDB
- SSH tunneling — connect through bastion hosts via russh, local port forwarding with key/password auth, auto-cleanup on disconnect
- Result set size limits — configurable max rows per query (default 10K), truncation warning banner when results exceed limit
- QueryTab pagination — client-side pagination for query results using existing Pagination component
- Large cell text truncation — display-layer truncation (500 chars) with character count badge for long text/JSON values
- Connection pool tuning — configurable pool size (1-50), idle timeout (10-3600s), acquire timeout (5-60s) per connection in Advanced section
- Backend cursor/streaming — server-side query pagination with `execute_query_page` and `count_query_rows` commands, automatic transition from client-side to server-side pagination when results are truncated, LIMIT/OFFSET wrapping with dialect-aware SQL (MSSQL OFFSET...FETCH vs standard LIMIT...OFFSET)
- Lazy column loading — large TEXT/JSON/Binary values truncated at configurable threshold (default 256 chars), `LargeText`/`LargeJson`/`LargeBinary` cell variants with preview + full_length, on-demand `fetch_full_cell` command to load full value, expand button in GridCell with size badge
- Max cell preview size setting — configurable truncation threshold (64-10000 chars) in Settings modal, passed to all query execution commands
- CI/CD pipeline — GitHub Actions workflows for CI checks (push/PR) and tag-triggered release builds across macOS (ARM + Intel), Windows, and Linux
- Auto-updater — in-app update notifications via tauri-plugin-updater, download + install + relaunch from status bar, GitHub Releases endpoint
- Installer packaging — .dmg (macOS), .exe/.msi (Windows), .deb/.rpm/.AppImage (Linux) via tauri-action
- Code signing ready — workflows pass through signing secrets; signs automatically when GitHub secrets are configured
- Schema selector — dropdown to pick active schema, multi-schema visibility with checkbox list, default schema persisted per connection
- Schema search path display — shows effective search_path (PostgreSQL), current database (MySQL), or default schema (MSSQL) in sidebar
- Schema context menu — right-click schema nodes to Create Schema (inline input), Drop Schema (with confirmation), or New Query Here (with schema context prefix)
- Cross-schema queries — schema-qualified tab titles when opening tables from non-active schemas, `getActiveSchema()` helper for determining default schema
- Startup optimization — parallelized store initialization with `Promise.all`, extracted window geometry restore, parallel session restore
- Schema cache LRU eviction — bounded detail cache (columns/indexes/FKs/stats) at 200 entries per connection, lazy eviction on cache writes
- Connection keepalive & auto-reconnect — 30s heartbeat ping, exponential backoff reconnection (3 retries: 2s/4s/8s), automatic schema reload on reconnect
- Virtual scrolling for schema tree — `VirtualTreeList` component virtualizes flat lists above 100 items, buffer rows for smooth scrolling, zero overhead below threshold
- Stress testing harness — Docker Compose + seed SQL generating 5 schemas x 400 tables x 10 columns (2000 tables) with indexes, FKs, views, functions, sequences, enums
- Multi-cell selection — click-drag rectangular cell range like Excel, Shift+click extend, keyboard arrow navigation with Shift+Arrow range extend, Ctrl+C copies cell range as TSV
- Column-level copy — Ctrl+click column header selects entire column, right-click column header for "Copy column values" / "Copy column with header"
- Paste from Excel/CSV — Ctrl+V parses TSV/CSV from clipboard and applies to cells starting at active cell, batch undo support in bulk edit mode
- Drag-fill cells — fill handle at bottom-right of selection, drag down/right to fill with detected pattern (repeat, arithmetic sequence, or cycle)
- Find & replace in grid — Ctrl+F search bar with match highlighting, Ctrl+H replace mode, case-sensitive toggle, Replace/Replace All for editable grids
- FK dropdowns — when editing a foreign key column, dropdown shows referenced values fetched via SELECT DISTINCT, with search filtering and keyboard navigation
- ER diagram viewer — interactive SVG canvas showing tables with columns and FK relationship lines, pan/zoom/drag, table selection highlights connected FKs, fit-to-screen, export SVG, schema filter, minimap navigation
- Visual query builder — drag-and-drop interface for building SELECT queries with JOINs/WHERE/GROUP BY/ORDER BY, column checkboxes for SELECT, join creation by clicking columns, real-time SQL preview, run results in inline grid
- Table structure diff — compare columns/indexes/foreign keys between two connections/schemas, color-coded added/removed/changed status, detailed change descriptions
- Migration generator — generates dialect-aware ALTER TABLE statements from table diff results (PostgreSQL, MySQL, MSSQL, SQLite), open in query tab or copy to clipboard
- Data diff — compare row data across two connections matched by primary key, side-by-side source/target display, changed cell highlighting, filter by status (added/removed/changed/identical)

### Stub databases (feature-gated, not yet functional):
- Oracle (`cargo build --features oracle` — requires Oracle Instant Client)

### Known issues to fix:
- ~~**SQL injection risk**: update_cell, insert_row, delete_rows use string concatenation instead of parameterized queries~~ (fixed: shared escape module with comprehensive literal escaping + identifier validation; ClickHouse/Snowflake/BigQuery drivers and schema.rs helpers hardened)
- ~~**Passwords stored in plaintext**: plugin-store saves JSON to disk unencrypted~~ (fixed: keychain-by-default for new connections, auto-migration of existing plaintext passwords on startup, all secret types — password, SSH password/passphrase, AWS secret key, GCP credentials — stored in OS keychain)
- ~~Connection pool hardcoded to 5, no idle timeout or health checks~~ (fixed: configurable pool tuning)
- ~~No query timeout enforcement~~ (fixed: configurable query timeout)
- ~~Full result sets loaded into memory (no streaming)~~ (mitigated: result set size limits with configurable max rows)
- ~~Schema cache never auto-invalidates~~ (fixed: auto-invalidate on DDL in QueryTab, clear table stats after DML in TableTab, command palette refresh event wired up)

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

### Phase 4: Data Export & Import ✅
- [x] **Export to CSV**: Export current result set or table data
- [x] **Export to JSON**: Export as JSON array
- [x] **Export to SQL**: Generate INSERT statements
- [x] **Export DDL**: Generate CREATE TABLE statements
- [x] **Import CSV**: Bulk load data from CSV files
- [x] **Copy as**: Copy selected rows as CSV/JSON/INSERT/Markdown

### Phase 5: Schema Browser Improvements ✅
- [x] **Views**: Show views separately from tables in the tree
- [x] **Functions/Procedures**: List stored functions and procedures
- [x] **Sequences**: Show sequences (PostgreSQL)
- [x] **Enums/Types**: Show custom types (PostgreSQL)
- [x] **Search**: Filter/search within the schema tree
- [x] **Table stats**: Show row count and size in tree tooltips
- [x] **Refresh**: Manual and auto-refresh for schema tree

### Phase 6: Advanced Database Operations ✅
- [x] **Transaction support**: BEGIN/COMMIT/ROLLBACK controls in the UI
- [x] **Query plan visualization**: EXPLAIN ANALYZE with visual tree
- [x] **Bulk editing**: Edit multiple cells/rows before committing
- [x] **Undo/redo for data changes**: Track changes locally before flushing
- [x] **Table creation**: GUI for creating new tables
- [x] **Table alteration**: Add/modify/drop columns through UI
- [x] **Index management**: Create/drop indexes through UI

### Phase 7: UX Polish ✅
- [x] **Connection groups/folders**: Organize connections
- [x] **Light theme**: Add light theme option with theme toggle
- [x] **Keyboard shortcuts panel**: Show all shortcuts, allow customization
- [x] **Tab drag-and-drop**: Reorder tabs by dragging
- [x] **Split panes**: View multiple tabs side by side
- [x] **Global search**: Ctrl+P to search tables, queries, connections

### Phase 8: Complete Stub Databases ✅
- [x] **Snowflake**: Full driver using snowflake-api crate (REST + Arrow), schema browsing via SHOW commands, DML via standard SQL
- [x] **BigQuery**: Full driver using gcp-bigquery-client crate (REST), schema browsing via client API, DML via standard SQL

### Phase 9: Global Shortcuts & Tab Management ✅
- [x] **Global keyboard shortcuts**: Ctrl+N (new query tab), Ctrl+W (close tab), Ctrl+Tab/Ctrl+Shift+Tab (cycle tabs), Ctrl+S (save query), F5 (refresh schema), Ctrl+B (toggle sidebar)
- [x] **Tab context menu**: Right-click tabs for Close, Close Others, Close All, Duplicate, Pin/Unpin
- [x] **Tab pinning**: Pin tabs to prevent accidental closure, show pin icon, hide close button on pinned tabs
- [x] **Sidebar collapse**: Toggle sidebar visibility with Ctrl+B or toolbar button, smooth collapse/expand

### Phase 10: Settings & Connection Enhancements ✅
- [x] **Settings/Preferences modal**: Configurable editor font size, grid font size, default page size, confirm-before-delete toggle — persisted via plugin-store
- [x] **Connection color coding**: Assign colors to connections, show colored border in sidebar and colored stripe on tabs
- [x] **Connection duplication**: Right-click connection → Duplicate creates a copy with "(copy)" suffix

### Phase 11: DDL Viewer & Type-Aware Editing ✅
- [x] **DDL/Source viewer**: View CREATE TABLE DDL for any table in a dedicated sub-tab with read-only CodeMirror and copy button (PostgreSQL, MySQL, SQLite, MSSQL, ClickHouse + wrapper drivers)
- [x] **Type-aware cell editing**: Boolean checkbox toggle, JSON textarea with monospace editing, auto-textarea for long text, NULL pill badge display
- [x] **NULL badge styling**: Replace italic NULL text with a styled pill badge for better visibility

### Phase 12: Query Cancellation ✅
- [x] **Query cancellation backend**: Cancellation tokens using tokio::select! with oneshot channels, QueryCancelled error variant
- [x] **Cancel button UI**: Show Cancel button next to spinner during query execution, call cancel_query command on click

### Phase 13: Production Readiness (partial) ✅
- [x] **Auto-update mechanism**: Integrate @tauri-apps/plugin-updater for in-app update notifications and automatic downloads
- [x] **Window state persistence**: Remember window size, position, sidebar width, and open tabs on restart using plugin-store
- [x] **Session restore**: Reopen last active tabs and connection on app launch
- [x] **Confirmation dialogs for destructive actions**: Confirm before DROP TABLE, bulk DELETE rows, disconnect with unsaved changes, close tabs with unsaved queries
- [x] **Empty states & onboarding**: First-launch experience with "Connect your first database" prompt, empty state illustrations for no connections/no tabs/no results
- [x] **About screen**: Version info, license, links to repo/docs/support
- [x] **App icon & branding**: Production app icon (navy + cyan magnifying glass over database grid), QueryArk rename across all files, keychain migration fallback

### Phase 14: Secure Connections ✅
- [x] **SSH tunneling**: Connect through SSH tunnel (local port forwarding) via russh, UI fields for SSH host/port/user/key/password, TunnelManager with bidirectional TCP forwarding
- [x] **SSL certificate configuration**: CA cert, client cert, client key file pickers with Tauri file dialog for PostgreSQL, MySQL, MariaDB, CockroachDB, Redshift (sqlx URL params) and MongoDB (TlsOptions)
- [x] **Connection URL input**: Parse connection strings (postgres://, mysql://, mongodb://, redis://, bolt://, sqlite:) and auto-fill form fields
- [x] **Keychain integration**: Store passwords in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) via keyring crate, lock icon toggle on password field

### Phase 15: Large Dataset Handling ✅
- [x] **Backend cursor/streaming**: Server-side query pagination via `execute_query_page` and `count_query_rows` commands, automatic transition when results are truncated, dialect-aware LIMIT/OFFSET wrapping (MSSQL OFFSET...FETCH vs standard)
- [x] **Virtual scrolling**: Render only visible rows in the data grid using a virtual scroll container (ROW_HEIGHT=32, BUFFER_ROWS=10, CSS translateY), supporting 100K+ rows without DOM bloat
- [x] **Lazy column loading**: Large TEXT/JSON/Binary values truncated at configurable threshold (default 256 chars), `LargeText`/`LargeJson`/`LargeBinary` cell variants with preview + full_length, on-demand `fetch_full_cell` command, expand button in GridCell
- [x] **Result set size limits**: Configurable max rows per query (default 10K), truncation warning when result exceeds limit, setting in Settings modal (100–100K)
- [x] **QueryTab pagination**: Client-side pagination for query results with page size selector, server-side pagination for truncated results
- [x] **Large cell text truncation**: Display-layer truncation (500 chars) with character count badge for long text/JSON cells
- [x] **Connection pool tuning**: Configurable pool size (1–50), idle timeout (10–3600s), acquire timeout (5–60s) per connection in Advanced section of connection modal

### Phase 16: Build & Distribution ✅
- [x] **GitHub Actions CI/CD**: Automated build pipeline for macOS (ARM + Intel), Windows (x64), and Linux (x64) — `.github/workflows/ci.yml`
- [x] **Code signing — macOS**: Workflow passes signing env vars through; signs automatically when GitHub secrets are configured
- [x] **Code signing — Windows**: Workflow passes signing env vars through; signs automatically when GitHub secrets are configured
- [x] **Installer packaging — macOS**: DMG via tauri-action (ARM + Intel separate builds)
- [x] **Installer packaging — Windows**: NSIS installer (.exe) + MSI via tauri-action
- [x] **Installer packaging — Linux**: .deb, .rpm, .AppImage via tauri-action
- [x] **Auto-updater backend**: GitHub Releases endpoint with `latest.json` manifest, tauri-plugin-updater integrated with status bar UI
- [x] **Release automation**: Tag-triggered builds via `.github/workflows/release.yml`, draft GitHub Release with all platform artifacts

### Phase 17: Product Website
Separate repository — SvelteKit static site deployed to Vercel/Netlify/Cloudflare Pages.

- [ ] **Landing page**: Hero section with tagline, app screenshot, and primary download CTA button that auto-detects visitor's OS (macOS/Windows/Linux)
- [ ] **Feature showcase**: Sections highlighting key features — multi-database support (17 engines), query editor, data grid, inline editing, schema browser — with screenshots/GIFs
- [ ] **Database grid**: Visual grid showing all 17 supported databases with icons and badges (similar to tableplus.com's supported databases section)
- [ ] **Download page**: Dedicated /download page with all platform options listed — macOS (DMG), Windows (MSI, portable), Linux (AppImage, .deb, .rpm, .pacman, Flatpak, Snap) — with version number and file sizes
- [ ] **Changelog page**: /changelog with version history, pulled from GitHub Releases or a markdown file
- [ ] **Documentation**: /docs with getting started guide, connection setup per database, keyboard shortcuts reference, FAQ
- [ ] **Pricing page**: /pricing if monetizing — free tier vs paid, feature comparison table, license purchase integration (Gumroad/Paddle/Stripe)
- [ ] **SEO & metadata**: Open Graph tags, Twitter cards, structured data, sitemap.xml, robots.txt
- [ ] **Analytics**: Privacy-friendly analytics (Plausible/Umami) for download counts and page views
- [ ] **Responsive design**: Mobile-friendly layout for all pages, dark theme matching the app aesthetic

### Phase 18: Performance & Reliability ✅
- [x] **Stress testing with large schemas**: Docker Compose + seed SQL (5 schemas × 400 tables = 2000 tables) for testing sidebar performance
- [x] **Virtual scrolling for schema tree**: VirtualTreeList component renders only visible tree nodes when lists exceed 100 items
- [x] **Connection keepalive & auto-reconnect**: 30s heartbeat ping, exponential backoff reconnection (3 retries), automatic schema reload
- [x] **Schema cache LRU eviction**: Bounded detail cache at 200 entries per connection, prevents unbounded memory growth
- [x] **Startup time optimization**: Parallelized store initialization and window/session restore with Promise.all

### Phase 19: Data Editing UX Polish ✅
- [x] **Multi-cell selection**: Click-drag range select like a spreadsheet
- [x] **Drag-fill cells**: Drag corner handle to fill pattern down/across
- [x] **Paste from Excel/CSV into grid**: Parse clipboard tabular data and insert into cells
- [x] **Find & replace within result set**: Search and replace values in the current grid view
- [x] **Column-level copy**: Click column header to select and copy entire column
- [x] **Foreign key dropdowns**: Show referenced values as dropdown options when editing FK columns

### Phase 20: Visual Database Tools ✅
- [x] **ER diagram viewer**: Interactive entity-relationship diagram showing table relationships with zoom/pan
- [x] **Visual query builder**: Drag-and-drop interface for constructing JOIN/WHERE/GROUP BY queries
- [x] **Table diff**: Compare table structure between two connections (dev vs prod)
- [x] **Data diff**: Compare row data between environments, highlight differences
- [x] **Migration generator**: Generate ALTER statements from schema diff between two databases

### Phase 21: Collaboration & Workflow
- [ ] **Connection config export/import**: Export connection bundles as encrypted JSON, share with team
- [ ] **Query snippets library**: Reusable parameterized query templates with variable substitution
- [ ] **Favorite tables**: Quick-access pinned tables per connection in sidebar
- [ ] **Query result bookmarks**: Save result snapshots with name and timestamp for later reference
- [ ] **Workspace profiles**: Save and restore layout + open tabs + connection per project

### Phase 22: Advanced Query Features
- [ ] **Query result charts**: Generate bar, line, pie charts from SELECT result sets
- [ ] **Side-by-side result comparison**: Compare two query results in split view with diff highlighting
- [ ] **Parameterized query execution**: Prompt for variable inputs ($1, :name) before running queries
- [ ] **Query profiling dashboard**: Execution time breakdown, index usage hints, optimization suggestions
- [ ] **Auto-suggest indexes**: Analyze slow queries and recommend index creation

### Phase 23: Quality & Trust
- [ ] **End-to-end test suite**: Playwright tests against Docker-based test databases (PostgreSQL, MySQL, SQLite, MongoDB, Redis)
- [ ] **Crash reporting**: Opt-in crash reporting via Sentry or similar service
- [ ] **Anonymous telemetry**: Opt-in usage analytics for feature prioritization (Plausible/PostHog)
- [ ] **In-app "What's New"**: Show changelog highlights after auto-update
- [ ] **Config auto-backup**: Automatically back up connection configs to timestamped files

### Phase 24: Schema Navigation & Management ✅
- [x] **Active schema selector**: Dropdown at the top of the schema tree to pick the active/default schema, filters tree to show only selected schema(s)
- [x] **Multi-schema visibility**: Checkbox list to show/hide specific schemas (like pgAdmin), persisted per connection
- [x] **Default schema per connection**: Remember last-selected schema per connection, auto-expand on reconnect
- [x] **Schema search path display**: Show the effective search_path (PostgreSQL) or current database/schema context in the sidebar header
- [x] **Schema creation/deletion**: Right-click context menu to CREATE SCHEMA / DROP SCHEMA with confirmation dialog
- [x] **Cross-schema queries**: Auto-prefix table references with schema name when querying outside the active schema

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

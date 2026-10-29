---
name: wasi-postgres
description: Access PostgreSQL from DJMXCreation's servers — native binaries and WASI 0.3 (P3 async) components — and tests with the latest wasi-pg-client. Use when writing repositories in the repository crate, SQL queries, row mapping, transactions, Postgres error mapping, SQL migrations in sql/migrations, or the migration runner. Do not use sqlx, diesel, or tokio-postgres.
---

# PostgreSQL

The only database client is **`wasi-pg-client`** (crates.io). It speaks the
Postgres wire protocol over WASI 0.3 (P3) async sockets on `wasm32-wasip2`
and over tokio on native (`tokio-transport` feature). The same repository
code compiles for both targets.

## Version

Always use the **latest release** (0.2.1 or newer). Check with
`cargo info wasi-pg-client` and bump it through `maintain-djmxcreation`.
From 0.2.1 on, the wasm transport needs the P3 async executor, which every
component provides through `wasip3::exports::http::handler::Guest` (see
`wasi-backend-api`). Two consequences:

- The workspace `wasip3` must be the version `wasi-pg-client` depends on
  (`cargo tree -i wasip3 --target wasm32-wasip2` shows one version).
- Components are served with
  `-W component-model-async=y -S p3=y -S tcp=y -S allow-ip-name-lookup=y`.

Never pin an older release to keep a P2 (`wstd`) entry point alive; move the
entry point to P3 instead.

## Dependencies (`repository/Cargo.toml`)

```toml
[dependencies]
wasi-pg-client = { workspace = true }   # uuid, serde-json, chrono set at workspace level
app-http = { workspace = true }         # timeout() on both targets

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
wasi-pg-client = { workspace = true, features = ["tokio-transport"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { workspace = true }
```

Despite its name, `wasi-pg-client` is the client for **both** runtimes. Wrap
connects and long queries in `app_http::timeout(duration, error, work)`; it
uses tokio natively and the P3 clock in a component, so repository code
needs no `cfg`.

## Connection model

**Open one connection per request or unit of work, and let it drop at the
end.** A WASI component cannot keep a pool between requests, and native
servers follow the same model so both runtimes behave alike. Pooling belongs
outside the server (PgBouncer). TLS to the database is handled by the
network/host layer.

```rust
/// Holds the PostgreSQL connection URL.
#[derive(Clone)]
pub struct DatabaseConfig { pub url: String }

impl DatabaseConfig {
    pub async fn connect(&self) -> Result<Connection, PgError> {
        let cfg = Config::from_uri(&self.url)
            .map_err(|e| PgError::Config(e.to_string()))?;
        Connection::connect(&cfg).await
    }
}
```

Repositories hold `Arc<DatabaseConfig>` and call `self.config.connect()`
inside each method. This is the house style for this project.

When several statements must commit together **across repositories**, add a
method that takes `conn: &mut Connection` next to the self-connecting one,
and let the service open one connection and run them in one transaction. Do
not mix the two styles inside one aggregate.

## Queries

- **Always parameterise**: `$1, $2, …`. Never use `format!` to build values
  into SQL. `format!` is acceptable only for constant column lists such as
  `const PROJECT_COLUMNS: &str`.
- Escape `%`, `_`, and `\` in user search text used with `LIKE`/`ILIKE`
  (an `escape_like` helper in `repository`).
- Use `RETURNING …` instead of a second `SELECT`.
- Paginate with `LIMIT $n OFFSET $m` and return `{ items, total }`. Bound the
  page size at the API boundary (maximum 100).
- Soft-delete only when the domain requires it (`deleted_at IS NULL`
  filters).

```rust
async fn get_project_by_id(&self, id: Uuid) -> Result<Option<ProjectDto>, Error> {
    let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
    let row = conn
        .query_params("SELECT * FROM project WHERE id = $1", &[&id])
        .await
        .map_err(|e| to_error(e, None))?
        .into_rows()
        .into_iter()
        .next();
    row.map(|r| Self::map_row(&r).map(ProjectDto::from)).transpose()
}
```

## Transactions

```rust
let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
let row = conn
    .with_transaction(async |txn| {
        txn.query_params(sql, &[&metadata, &now])
            .await?
            .into_rows()
            .into_iter()
            .next()
            .ok_or_else(|| PgError::UnexpectedNull { column: "id".into() })
    })
    .await
    .map_err(|e| to_error(e, None))?;
```

`with_transaction` begins, commits on `Ok`, and rolls back on `Err`.
Consuming one-shot state (upload sessions, media-processing job claims) must
be atomic: a conditional `UPDATE … RETURNING` or a transaction with
`SELECT … FOR UPDATE SKIP LOCKED`.

## Row mapping

- Map **by column name** with small helpers. This is robust against column
  order:

```rust
fn col<T: wasi_pg_client::types::FromSql>(row: &Row, name: &str) -> Result<T, Error> {
    row.get_by_name(name).map_err(|e| to_error(e, None))
}
```

- Rows map to an **entity** (`repository::entity::<thing>`), which
  converts with `impl From<Entity> for Dto`. Domain types never see `Row`.
- Use `JSONB` columns for flexible documents (`serde_json::Value`: project
  metadata, content blocks), and typed columns for everything you filter,
  sort, or join on.

## Error mapping (`repository/src/error.rs`)

`to_error(PgError, Option<String>) -> app_core::Error`, using
`wasi_pg_client::error::sqlstate`:

| SQLSTATE | Domain error |
|---|---|
| `UNIQUE_VIOLATION` (23505) | `Conflict("already exists")` |
| `FOREIGN_KEY_VIOLATION` | `InvalidInput("referenced entity not found")` |
| `NOT_NULL_VIOLATION`, `INVALID_PARAMETER_VALUE` | `InvalidInput` |
| `error.is_connection_broken()` | `Database` (log at `error`) |
| other | `Database` (log at `error`, never expose the message) |

Callers never match on SQL error strings. Extend `to_error` when a caller
needs a new distinction.

## Migrations

Files: `sql/migrations/V<n>__<UPPER_SNAKE_DESCRIPTION>.sql` (for example
`V3__CREATE_TABLE_PROJECT.sql`). Rules:

- Versions are unique and only increase. **Never edit an applied migration**;
  add a new one with the next free number (check the directory; numbers may
  have gaps).
- Make statements idempotent where cheap (`CREATE TABLE IF NOT EXISTS`,
  `ADD COLUMN IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`).
- `uuid` primary keys (generated in Rust with v4, or `gen_random_uuid()`),
  `timestamptz` columns `created_on`/`updated_on`, and explicit foreign keys
  with `ON DELETE` behaviour.
- Add an index for every foreign key and every filtered or sorted column.
- Store object-storage **keys**, never URLs.

Runner crate `migration` (native lib + bin):

- Reads `sql/migrations`, parses `V<n>`, rejects duplicate versions, and
  sorts numerically.
- Tracks applied files in `_migrations(filename UNIQUE, applied_at)`.
- Takes `pg_advisory_lock(<constant>)` with a `lock_timeout` so concurrent
  runners are safe.
- Applies each file with `batch_execute` inside `BEGIN`/`COMMIT` and records
  it in `_migrations` in the same transaction.
- Is exposed as `pub async fn run_migrations(database_url: &str)` so
  `dev-server` and `test-util` reuse it.

Servers **never** migrate at startup, native or WASI. Migrations run before a deploy
(`DATABASE_URL=… cargo run -p migration`), and `dev-server` and
`test-util` run them for local and test databases.

## Testing

Repository tests run against a real Postgres started once per test process
by `test-util` (rustainers + Podman), with migrations applied. Each test
creates its own rows with fresh UUIDs. See `wasi-testing`.

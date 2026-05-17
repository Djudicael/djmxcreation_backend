//! Database migration library.
//!
//! Reads SQL files from `sql/migrations/` and applies them in order.
//! Used by both the standalone `migration` binary and the `dev-server` orchestrator.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn parse_migration_version(name: &str) -> anyhow::Result<u32> {
    name.split_once('_')
        .and_then(|(prefix, _)| prefix.strip_prefix('V'))
        .and_then(|num| num.parse::<u32>().ok())
        .ok_or_else(|| anyhow::anyhow!("invalid migration filename format: {name}"))
}

fn ensure_unique_migration_versions<'a>(
    filenames: impl IntoIterator<Item = &'a str>,
) -> anyhow::Result<()> {
    let mut versions: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for name in filenames {
        let version = parse_migration_version(name)?;
        versions.entry(version).or_default().push(name.to_string());
    }

    let duplicate_versions: Vec<String> = versions
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(version, files)| format!("V{version}: {}", files.join(", ")))
        .collect();
    if !duplicate_versions.is_empty() {
        anyhow::bail!(
            "duplicate migration versions detected; each V<number> prefix must be unique:\n{}",
            duplicate_versions.join("\n")
        );
    }

    Ok(())
}

fn migration_sort_key(name: &str) -> anyhow::Result<(u32, String)> {
    Ok((parse_migration_version(name)?, name.to_string()))
}

/// Run all SQL migration files from `sql/migrations/` against the given database URL.
///
/// Files are sorted by their `V<number>__` prefix (V1, V2, ..., V10, ...).
/// Already-applied migrations are tracked in a `_migrations` table and skipped.
pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    let config = wasi_pg_client::Config::from_uri(database_url)?;
    let mut conn = wasi_pg_client::Connection::connect(&config).await?;

    let migrations_dir = Path::new("sql/migrations");
    if !migrations_dir.exists() {
        anyhow::bail!(
            "migrations directory not found: {}",
            migrations_dir.display()
        );
    }

    conn.query(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            id SERIAL PRIMARY KEY,
            filename TEXT NOT NULL UNIQUE,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .await?;

    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            s.ends_with(".sql")
        })
        .collect();

    let migration_names: Vec<String> = entries
        .iter()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    ensure_unique_migration_versions(migration_names.iter().map(String::as_str))?;

    entries.sort_by_key(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        migration_sort_key(&name).unwrap_or((0, name))
    });

    for entry in entries {
        let filename = entry.file_name().to_string_lossy().to_string();

        let check = conn
            .query_params(
                "SELECT 1 FROM _migrations WHERE filename = $1",
                &[&filename],
            )
            .await?;
        if !check.into_rows().is_empty() {
            tracing::info!("skip {filename} (already applied)");
            continue;
        }

        let path = entry.path();
        let sql = fs::read_to_string(&path)?;

        tracing::info!("apply {filename}");
        conn.batch_execute(&sql)
            .await
            .map_err(|e| anyhow::anyhow!("failed to apply {}: {}", filename, e))?;

        conn.execute_params(
            "INSERT INTO _migrations (filename) VALUES ($1)",
            &[&filename],
        )
        .await?;

        tracing::info!("applied {filename} ok");
    }

    tracing::info!("migrations complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_migration_version_accepts_valid_names() {
        assert_eq!(parse_migration_version("V1__CREATE_BDD.sql").unwrap(), 1);
        assert_eq!(parse_migration_version("V27__add_feature.sql").unwrap(), 27);
    }

    #[test]
    fn parse_migration_version_rejects_invalid_names() {
        assert!(parse_migration_version("init.sql").is_err());
        assert!(parse_migration_version("Vx__bad.sql").is_err());
    }

    #[test]
    fn migration_sort_key_uses_numeric_version_order() {
        let mut names = vec![
            "V10__later.sql".to_string(),
            "V2__second.sql".to_string(),
            "V1__first.sql".to_string(),
        ];

        names.sort_by_key(|name| migration_sort_key(name).unwrap());

        assert_eq!(
            names,
            vec![
                "V1__first.sql".to_string(),
                "V2__second.sql".to_string(),
                "V10__later.sql".to_string(),
            ]
        );
    }
}

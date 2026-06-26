use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(crate) const CONFIG_FILE_NAME: &str = "caglla.toml";
pub(crate) const DEFAULT_DB_FILE: &str = "caglla.db";
pub(crate) const ENV_DB_PATH: &str = "CAGLLA_DB";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbPathSource {
    Cli,
    Env,
    Config,
    Default,
}

impl DbPathSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Env => "env",
            Self::Config => "config",
            Self::Default => "default",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedDbPath {
    pub path: PathBuf,
    pub source: DbPathSource,
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbPathResolveInputs<'a> {
    pub cwd: &'a Path,
    pub cli_db: Option<&'a Path>,
    pub env_caglla_db: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct CagllaConfigFile {
    database: DatabaseSection,
}

#[derive(Debug, Deserialize)]
struct DatabaseSection {
    path: String,
}

pub(crate) fn resolve_db_path_for_cli(cli_db: Option<&Path>) -> Result<ResolvedDbPath> {
    let cwd = std::env::current_dir().context("作業ディレクトリの取得に失敗しました")?;
    resolve_db_path(DbPathResolveInputs {
        cwd: &cwd,
        cli_db,
        env_caglla_db: std::env::var(ENV_DB_PATH).ok().as_deref(),
    })
}

pub(crate) fn resolve_db_path(inputs: DbPathResolveInputs<'_>) -> Result<ResolvedDbPath> {
    if let Some(cli_path) = inputs.cli_db {
        return Ok(ResolvedDbPath {
            path: resolve_user_path(inputs.cwd, cli_path)?,
            source: DbPathSource::Cli,
            config_path: None,
        });
    }

    if let Some(env_path) = inputs.env_caglla_db {
        if !env_path.trim().is_empty() {
            return Ok(ResolvedDbPath {
                path: resolve_user_path(inputs.cwd, Path::new(env_path))?,
                source: DbPathSource::Env,
                config_path: None,
            });
        }
    }

    let config_path = inputs.cwd.join(CONFIG_FILE_NAME);
    if config_path.is_file() {
        let configured = read_database_path_from_config(&config_path)?;
        let config_dir = config_path
            .parent()
            .context("config ファイルの親ディレクトリを解決できませんでした")?;
        return Ok(ResolvedDbPath {
            path: resolve_user_path(config_dir, Path::new(&configured))?,
            source: DbPathSource::Config,
            config_path: Some(config_path),
        });
    }

    Ok(ResolvedDbPath {
        path: inputs.cwd.join(DEFAULT_DB_FILE),
        source: DbPathSource::Default,
        config_path: None,
    })
}

fn read_database_path_from_config(config_path: &Path) -> Result<String> {
    let contents = std::fs::read_to_string(config_path).with_context(|| {
        format!(
            "config ファイル '{}' の読み込みに失敗しました",
            config_path.display()
        )
    })?;
    let parsed: CagllaConfigFile = toml::from_str(&contents).with_context(|| {
        format!(
            "config ファイル '{}' の TOML 解析に失敗しました",
            config_path.display()
        )
    })?;
    let path = parsed.database.path.trim().to_string();
    if path.is_empty() {
        bail!(
            "config ファイル '{}' の [database].path が空です",
            config_path.display()
        );
    }
    Ok(path)
}

fn resolve_user_path(base: &Path, path: &Path) -> Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    Ok(normalize_path(joined))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    use std::path::Component;
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component);
                }
            }
            other => normalized.push(other),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestWorkdir(std::path::PathBuf);

    impl TestWorkdir {
        fn new() -> Self {
            let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("travel-ledger-cli-config-test-{n}"));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestWorkdir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_config(dir: &Path, contents: &str) {
        fs::write(dir.join(CONFIG_FILE_NAME), contents).unwrap();
    }

    #[test]
    fn resolve_cli_path_has_highest_priority() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./from-config.db\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: Some(Path::new("./from-cli.db")),
            env_caglla_db: Some("./from-env.db"),
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Cli);
        assert_eq!(resolved.path, dir.path().join("from-cli.db"));
        assert!(resolved.config_path.is_none());
    }

    #[test]
    fn resolve_env_path_beats_config_and_default() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./from-config.db\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: Some("./from-env.db"),
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Env);
        assert_eq!(resolved.path, dir.path().join("from-env.db"));
    }

    #[test]
    fn resolve_config_path_beats_default() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./from-config.db\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Config);
        assert_eq!(resolved.path, dir.path().join("from-config.db"));
        assert_eq!(
            resolved.config_path,
            Some(dir.path().join(CONFIG_FILE_NAME))
        );
    }

    #[test]
    fn resolve_default_uses_cwd_caglla_db() {
        let dir = TestWorkdir::new();
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Default);
        assert_eq!(resolved.path, dir.path().join(DEFAULT_DB_FILE));
    }

    #[test]
    fn resolve_config_relative_path_uses_config_dir() {
        let dir = TestWorkdir::new();
        fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            "[database]\npath = \"data/app.db\"\n",
        )
        .unwrap();
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.path, dir.path().join("data/app.db"));
    }

    #[test]
    fn resolve_invalid_toml_is_error() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "not = [valid");
        let err = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap_err();
        assert!(err.to_string().contains("TOML"));
    }

    #[test]
    fn resolve_empty_database_path_is_error() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"\"\n");
        let err = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap_err();
        assert!(err.to_string().contains("path が空"));
    }
}

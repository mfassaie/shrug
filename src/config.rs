use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::cli::{ColorChoice, OutputFormat};
use crate::error::ShrugError;

/// Fully resolved configuration with sensible defaults for all fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShrugConfig {
    pub output_format: OutputFormat,
    pub color: ColorChoice,
    pub default_profile: Option<String>,
    pub site: Option<String>,
    pub page_size: u32,
    pub cache_ttl_hours: u32,
}

impl Default for ShrugConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Table,
            color: ColorChoice::Auto,
            default_profile: None,
            site: None,
            page_size: 50,
            cache_ttl_hours: 24,
        }
    }
}

/// Partial config for TOML deserialization. Absent fields stay None,
/// preventing silent reset of values from prior layers.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ShrugConfigPartial {
    output_format: Option<OutputFormat>,
    color: Option<ColorChoice>,
    default_profile: Option<String>,
    site: Option<String>,
    page_size: Option<u32>,
    cache_ttl_hours: Option<u32>,
}

impl ShrugConfig {
    /// Overlay non-None fields from a partial config onto this config.
    fn merge(&mut self, partial: ShrugConfigPartial) {
        if let Some(v) = partial.output_format {
            self.output_format = v;
        }
        if let Some(v) = partial.color {
            self.color = v;
        }
        if partial.default_profile.is_some() {
            self.default_profile = partial.default_profile;
        }
        if partial.site.is_some() {
            self.site = partial.site;
        }
        if let Some(v) = partial.page_size {
            self.page_size = v;
        }
        if let Some(v) = partial.cache_ttl_hours {
            self.cache_ttl_hours = v;
        }
    }

    /// Apply CLI flag overrides. Only overrides if the CLI provided a non-default value.
    pub fn apply_cli_overrides(
        &mut self,
        output: &OutputFormat,
        color: &ColorChoice,
        profile: &Option<String>,
    ) {
        self.output_format = output.clone();
        self.color = color.clone();
        if profile.is_some() {
            self.default_profile = profile.clone();
        }
    }
}

/// Platform-correct paths for shrug config, cache, and data directories.
pub struct ShrugPaths {
    dirs: ProjectDirs,
}

impl ShrugPaths {
    pub fn new() -> Option<Self> {
        ProjectDirs::from("", "", "shrug").map(|dirs| Self { dirs })
    }

    pub fn config_dir(&self) -> &Path {
        self.dirs.config_dir()
    }

    pub fn cache_dir(&self) -> &Path {
        self.dirs.cache_dir()
    }

    pub fn data_dir(&self) -> &Path {
        self.dirs.data_dir()
    }

    pub fn user_config_path(&self) -> PathBuf {
        self.dirs.config_dir().join("config.toml")
    }

    /// Walk up from cwd looking for .shrug.toml.
    /// Stops at the first directory containing .git, or at filesystem root.
    pub fn project_config_path() -> Option<PathBuf> {
        let cwd = env::current_dir().ok()?;
        let mut dir = cwd.as_path();

        loop {
            let candidate = dir.join(".shrug.toml");
            if candidate.is_file() {
                return Some(candidate);
            }

            // If this directory contains .git, stop searching
            if dir.join(".git").exists() {
                return None;
            }

            // Move to parent, stop at filesystem root
            match dir.parent() {
                Some(parent) => dir = parent,
                None => return None,
            }
        }
    }
}

/// Load a TOML config file into a partial config.
fn load_toml_file(path: &Path) -> Result<ShrugConfigPartial, ShrugError> {
    let content = fs::read_to_string(path).map_err(|e| {
        ShrugError::ConfigError(format!(
            "Failed to read config file {}: {}",
            path.display(),
            e
        ))
    })?;
    toml::from_str(&content).map_err(|e| {
        ShrugError::ConfigError(format!(
            "Invalid TOML in config file {}: {}",
            path.display(),
            e
        ))
    })
}

/// Apply environment variable overrides to the config.
fn apply_env_overrides(config: &mut ShrugConfig) -> Result<(), ShrugError> {
    if let Ok(val) = env::var("SHRUG_OUTPUT") {
        config.output_format = match val.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "table" => OutputFormat::Table,
            "csv" => OutputFormat::Csv,
            _ => {
                return Err(ShrugError::ConfigError(format!(
                    "Invalid value for SHRUG_OUTPUT: '{}'. Expected one of: json, table, csv",
                    val
                )));
            }
        };
    }

    if let Ok(val) = env::var("SHRUG_COLOR") {
        config.color = match val.to_lowercase().as_str() {
            "auto" => ColorChoice::Auto,
            "always" => ColorChoice::Always,
            "never" => ColorChoice::Never,
            _ => {
                return Err(ShrugError::ConfigError(format!(
                    "Invalid value for SHRUG_COLOR: '{}'. Expected one of: auto, always, never",
                    val
                )));
            }
        };
    }

    if let Ok(val) = env::var("SHRUG_PROFILE") {
        config.default_profile = Some(val);
    }

    if let Ok(val) = env::var("SHRUG_SITE") {
        config.site = Some(val);
    }

    if let Ok(val) = env::var("SHRUG_PAGE_SIZE") {
        config.page_size = val.parse::<u32>().map_err(|_| {
            ShrugError::ConfigError(format!(
                "Invalid value for SHRUG_PAGE_SIZE: '{}'. Expected a positive integer",
                val
            ))
        })?;
    }

    Ok(())
}

/// Load configuration with layered precedence:
/// defaults < user config < project config < env vars
pub fn load_config() -> Result<ShrugConfig, ShrugError> {
    let mut config = ShrugConfig::default();

    // Layer 1: User config file
    if let Some(paths) = ShrugPaths::new() {
        let user_path = paths.user_config_path();
        if user_path.is_file() {
            let partial = load_toml_file(&user_path)?;
            config.merge(partial);
        }
    }

    // Layer 2: Project config file
    if let Some(project_path) = ShrugPaths::project_config_path() {
        let partial = load_toml_file(&project_path)?;
        config.merge(partial);
    }

    // Layer 3: Environment variables
    apply_env_overrides(&mut config)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Env var tests must run serially to avoid race conditions.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn default_config_has_sensible_values() {
        let config = ShrugConfig::default();
        assert_eq!(config.output_format, OutputFormat::Table);
        assert_eq!(config.color, ColorChoice::Auto);
        assert!(config.default_profile.is_none());
        assert!(config.site.is_none());
        assert_eq!(config.page_size, 50);
        assert_eq!(config.cache_ttl_hours, 24);
    }

    #[test]
    fn partial_config_from_toml_with_subset_of_fields() {
        let toml_str = r#"
            output_format = "json"
            page_size = 100
        "#;
        let partial: ShrugConfigPartial = toml::from_str(toml_str).unwrap();
        assert_eq!(partial.output_format, Some(OutputFormat::Json));
        assert_eq!(partial.page_size, Some(100));
        assert!(partial.color.is_none());
        assert!(partial.site.is_none());
    }

    #[test]
    fn merge_overwrites_only_present_fields() {
        let mut config = ShrugConfig {
            page_size: 25, // Set by prior layer
            ..ShrugConfig::default()
        };

        let partial: ShrugConfigPartial = toml::from_str(r#"output_format = "json""#).unwrap();
        config.merge(partial);

        assert_eq!(config.output_format, OutputFormat::Json); // Overwritten
        assert_eq!(config.page_size, 25); // Preserved from prior layer
        assert_eq!(config.color, ColorChoice::Auto); // Preserved default
    }

    #[test]
    fn merge_precedence_project_over_user() {
        let mut config = ShrugConfig::default();

        // Simulate user config
        let user: ShrugConfigPartial =
            toml::from_str("output_format = \"json\"\npage_size = 30").unwrap();
        config.merge(user);

        // Simulate project config (only overrides output)
        let project: ShrugConfigPartial = toml::from_str(r#"output_format = "csv""#).unwrap();
        config.merge(project);

        assert_eq!(config.output_format, OutputFormat::Csv); // Project wins
        assert_eq!(config.page_size, 30); // User preserved (project didn't set)
    }

    #[test]
    fn invalid_toml_produces_config_error_with_path() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "this is not valid toml {{{{").unwrap();

        let result = load_toml_file(&path);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("bad.toml"),
            "Error should contain file path: {err_msg}"
        );
    }

    #[test]
    fn cli_override_applies() {
        let mut config = ShrugConfig {
            output_format: OutputFormat::Json, // From file
            ..ShrugConfig::default()
        };

        config.apply_cli_overrides(
            &OutputFormat::Table,
            &ColorChoice::Never,
            &Some("staging".into()),
        );

        assert_eq!(config.output_format, OutputFormat::Table);
        assert_eq!(config.color, ColorChoice::Never);
        assert_eq!(config.default_profile, Some("staging".into()));
    }

    #[test]
    fn project_config_search_stops_at_git_root() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        // Set cwd to nested dir — should NOT find .shrug.toml above .git
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&nested).unwrap();
        let result = ShrugPaths::project_config_path();
        env::set_current_dir(original_dir).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn project_config_found_in_git_root() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(dir.path().join(".shrug.toml"), "page_size = 99\n").unwrap();
        let nested = dir.path().join("src");
        fs::create_dir_all(&nested).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&nested).unwrap();
        let result = ShrugPaths::project_config_path();
        env::set_current_dir(original_dir).unwrap();

        assert!(result.is_some());
        assert!(result.unwrap().ends_with(".shrug.toml"));
    }

    // Env var tests use unique var names to avoid parallel test pollution
    #[test]
    fn env_var_invalid_page_size_returns_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = "SHRUG_PAGE_SIZE";
        let original = env::var(key).ok();
        env::set_var(key, "abc");

        let mut config = ShrugConfig::default();
        let result = apply_env_overrides(&mut config);

        // Restore
        match original {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("SHRUG_PAGE_SIZE"),
            "Error should name the var: {err_msg}"
        );
    }

    #[test]
    fn env_var_invalid_output_returns_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = "SHRUG_OUTPUT";
        let original = env::var(key).ok();
        env::set_var(key, "xml");

        let mut config = ShrugConfig::default();
        let result = apply_env_overrides(&mut config);

        match original {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("SHRUG_OUTPUT"),
            "Error should name the var: {err_msg}"
        );
    }

    #[test]
    fn env_var_valid_overrides_apply() {
        let _guard = ENV_LOCK.lock().unwrap();
        let keys = ["SHRUG_OUTPUT", "SHRUG_SITE", "SHRUG_PAGE_SIZE"];
        let originals: Vec<_> = keys.iter().map(|k| env::var(k).ok()).collect();

        env::set_var("SHRUG_OUTPUT", "json");
        env::set_var("SHRUG_SITE", "test.atlassian.net");
        env::set_var("SHRUG_PAGE_SIZE", "200");

        let mut config = ShrugConfig::default();
        let result = apply_env_overrides(&mut config);

        // Restore
        for (key, orig) in keys.iter().zip(originals) {
            match orig {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }

        assert!(result.is_ok());
        assert_eq!(config.output_format, OutputFormat::Json);
        assert_eq!(config.site, Some("test.atlassian.net".into()));
        assert_eq!(config.page_size, 200);
    }

    #[test]
    fn shrug_paths_creates_successfully() {
        let paths = ShrugPaths::new();
        assert!(paths.is_some());
        let paths = paths.unwrap();
        assert!(paths.config_dir().to_str().unwrap().contains("shrug"));
    }

    #[test]
    fn serialize_config_roundtrips() {
        let config = ShrugConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("output_format"));
        assert!(toml_str.contains("table"));
    }

    #[test]
    fn load_config_layered_precedence() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        // User config: output = json, page_size = 75
        let user_path = dir.path().join("user.toml");
        fs::write(&user_path, "output_format = \"json\"\npage_size = 75\n").unwrap();

        // Project config: output = table (overrides user)
        let project_path = dir.path().join("project.toml");
        fs::write(&project_path, "output_format = \"table\"\n").unwrap();

        // Load and merge: user first, then project on top
        let user_partial = load_toml_file(&user_path).unwrap();
        let project_partial = load_toml_file(&project_path).unwrap();

        let mut config = ShrugConfig::default();
        config.merge(user_partial);
        config.merge(project_partial);

        // Before env: project wins over user for output, user's page_size preserved
        assert_eq!(config.output_format, OutputFormat::Table);
        assert_eq!(config.page_size, 75);

        // Env var override: SHRUG_OUTPUT=csv wins over everything
        let orig = env::var("SHRUG_OUTPUT").ok();
        env::set_var("SHRUG_OUTPUT", "csv");
        let result = apply_env_overrides(&mut config);
        match orig {
            Some(v) => env::set_var("SHRUG_OUTPUT", v),
            None => env::remove_var("SHRUG_OUTPUT"),
        }
        result.unwrap();

        assert_eq!(config.output_format, OutputFormat::Csv); // env wins
        assert_eq!(config.page_size, 75); // user config preserved
        assert_eq!(config.cache_ttl_hours, 24); // default preserved
    }
}

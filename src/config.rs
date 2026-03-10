use anyhow::Result;
use directories::ProjectDirs;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub model: ModelConfig,
}

pub struct Ignore {
    inner: Gitignore,
}

impl Ignore {
    pub fn load() -> Self {
        let mut builder = GitignoreBuilder::new(".");

        // 1. System-wide ignore
        #[cfg(unix)]
        let system_path = Path::new("/etc/siftignore");
        #[cfg(windows)]
        let system_path = Path::new(r"C:\ProgramData\sift\siftignore");
        if system_path.exists() {
            builder.add(system_path);
        }

        // 2. User-specific ignore
        if let Some(proj_dirs) = ProjectDirs::from("com", "rupurt", "sift") {
            let user_path = proj_dirs.config_dir().join("siftignore");
            if user_path.exists() {
                builder.add(user_path);
            }
        }

        // 3. Local directory ignore
        let local_path = Path::new("./.siftignore");
        if local_path.exists() {
            builder.add(local_path);
        }

        // 4. Default exclusions for search quality (noise reduction)
        let default_exclusions = [
            ".keel/**",
            "target/**",
            ".git/**",
            ".direnv/**",
            "flake.lock",
        ];
        for pattern in default_exclusions {
            let _ = builder.add_line(None, pattern);
        }

        Self {
            inner: builder.build().unwrap_or_else(|_| Gitignore::empty()),
        }
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        self.inner.matched(path, path.is_dir()).is_ignore()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_shortlist")]
    pub shortlist: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default = "default_model_id")]
    pub model_id: String,
    #[serde(default = "default_model_revision")]
    pub model_revision: String,
    #[serde(default = "default_max_length")]
    pub max_length: usize,
}

fn default_strategy() -> String {
    "hybrid".to_string()
}
fn default_limit() -> usize {
    10
}
fn default_shortlist() -> usize {
    8
}
fn default_model_id() -> String {
    crate::dense::DEFAULT_MODEL_ID.to_string()
}
fn default_model_revision() -> String {
    crate::dense::DEFAULT_MODEL_REVISION.to_string()
}
fn default_max_length() -> usize {
    crate::dense::DEFAULT_MAX_LENGTH
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
            limit: default_limit(),
            shortlist: default_shortlist(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_id: default_model_id(),
            model_revision: default_model_revision(),
            max_length: default_max_length(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // 1. System-wide config
        #[cfg(unix)]
        let system_path = Path::new("/etc/sift.toml");
        #[cfg(windows)]
        let system_path = Path::new(r"C:\ProgramData\sift\config.toml");

        config.merge_file(system_path)?;

        // 2. User-specific config
        if let Some(proj_dirs) = ProjectDirs::from("com", "rupurt", "sift") {
            let user_path = proj_dirs.config_dir().join("sift.toml");
            config.merge_file(&user_path)?;
        }

        // 3. Local directory config
        let local_path = Path::new("./sift.toml");
        config.merge_file(local_path)?;

        Ok(config)
    }

    pub fn highlight_toml(input: &str) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        for line in input.lines() {
            if line.trim().starts_with('[') && line.trim().ends_with(']') {
                // Section header in Cyan
                writeln!(output, "\x1b[36m{}\x1b[0m", line).unwrap();
            } else if let Some((key, value)) = line.split_once('=') {
                // Key-value pair
                let key_part = key.trim_end();
                let value_part = value.trim();

                // Key in Bold Blue
                write!(output, "\x1b[1;34m{}\x1b[0m = ", key_part).unwrap();

                if value_part.starts_with('"') {
                    // String in Green
                    writeln!(output, "\x1b[32m{}\x1b[0m", value_part).unwrap();
                } else if value_part == "true" || value_part == "false" || value_part == "null" {
                    // Boolean/Null in Yellow
                    writeln!(output, "\x1b[33m{}\x1b[0m", value_part).unwrap();
                } else if !value_part.is_empty()
                    && value_part.chars().all(|c| c.is_numeric() || c == '.')
                {
                    // Number in Yellow
                    writeln!(output, "\x1b[33m{}\x1b[0m", value_part).unwrap();
                } else {
                    writeln!(output, "{}", value_part).unwrap();
                }
            } else {
                writeln!(output, "{}", line).unwrap();
            }
        }
        output
    }

    fn merge_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let partial: PartialConfig = toml::from_str(&content)?;

        if let Some(search) = partial.search {
            if let Some(strategy) = search.strategy {
                self.search.strategy = strategy;
            }
            if let Some(limit) = search.limit {
                self.search.limit = limit;
            }
            if let Some(shortlist) = search.shortlist {
                self.search.shortlist = shortlist;
            }
        }

        if let Some(model) = partial.model {
            if let Some(model_id) = model.model_id {
                self.model.model_id = model_id;
            }
            if let Some(model_revision) = model.model_revision {
                self.model.model_revision = model_revision;
            }
            if let Some(max_length) = model.max_length {
                self.model.max_length = max_length;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    search: Option<PartialSearchConfig>,
    model: Option<PartialModelConfig>,
}

#[derive(Debug, Deserialize)]
struct PartialSearchConfig {
    strategy: Option<String>,
    limit: Option<usize>,
    shortlist: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct PartialModelConfig {
    model_id: Option<String>,
    model_revision: Option<String>,
    max_length: Option<usize>,
}

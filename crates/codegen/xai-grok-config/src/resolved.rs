//! A resolved config value tagged with the layer it came from.

/// Where a resolved config value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum ConfigSource {
    Requirement,
    Cli,
    Env,
    SystemManagedConfig,
    ManagedConfig,
    UserConfig,
    /// A value injected via the `GROK_CONFIG` / `GROK_CONFIG_PATH` overlay.
    EnvOverlay,
    Config,
    Remote,
    Default,
}

/// A resolved config value with its source for diagnostics.
#[derive(Debug, Clone)]
pub struct Resolved<T> {
    pub value: T,
    pub source: ConfigSource,
}

impl<T> Resolved<T> {
    pub fn new(value: T, source: ConfigSource) -> Self {
        Self { value, source }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Resolved<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.value, self.source)
    }
}

/// Resolve a string setting: cli > env > config > feature flag.
/// `None` if no source provides a value.
pub fn resolve_string_flag(
    cli_arg: Option<&str>,
    env_var: &str,
    config_val: Option<&str>,
    feature_flag_val: Option<&str>,
) -> Option<Resolved<String>> {
    if let Some(val) = cli_arg.filter(|s| !s.is_empty()) {
        return Some(Resolved::new(val.to_owned(), ConfigSource::Cli));
    }
    if let Some(val) = xai_grok_env::env_string(env_var) {
        return Some(Resolved::new(val, ConfigSource::Env));
    }
    if let Some(val) = config_val.filter(|s| !s.is_empty()) {
        return Some(Resolved::new(val.to_owned(), ConfigSource::Config));
    }
    if let Some(val) = feature_flag_val.filter(|s| !s.is_empty()) {
        return Some(Resolved::new(val.to_owned(), ConfigSource::Remote));
    }
    None
}

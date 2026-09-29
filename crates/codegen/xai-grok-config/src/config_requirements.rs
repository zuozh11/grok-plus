use std::ops::Deref;

use serde::Deserialize;
use strum::IntoEnumIterator;

use crate::validation::RequirementsSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowlistPin {
    List(Vec<String>),
    FailClosed,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum::EnumIter,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum ToolFeature {
    AskUserQuestion,
    ImageEdit,
    ImageGen,
    LspTools,
    VideoGen,
    WebFetch,
    WriteFile,
}

impl ToolFeature {
    #[must_use]
    pub fn tool_names(self) -> &'static [&'static str] {
        match self {
            ToolFeature::AskUserQuestion => &["ask_user_question"],
            ToolFeature::ImageEdit => &["image_edit"],
            ToolFeature::ImageGen => &["image_gen"],
            ToolFeature::LspTools => &["lsp"],
            ToolFeature::VideoGen => &["image_to_video", "reference_to_video"],
            ToolFeature::WebFetch => &["web_fetch"],
            ToolFeature::WriteFile => &["write"],
        }
    }

    fn pin(self, parsed: &RequirementsToml) -> Option<bool> {
        match self {
            ToolFeature::AskUserQuestion => parsed.ask_user_question,
            ToolFeature::ImageEdit => parsed.image_edit,
            ToolFeature::ImageGen => parsed.image_gen,
            ToolFeature::LspTools => parsed.lsp_tools,
            ToolFeature::VideoGen => parsed.video_gen,
            ToolFeature::WebFetch => parsed.web_fetch,
            ToolFeature::WriteFile => parsed.write_file,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "RequirementsFile")]
pub struct RequirementsToml {
    pub allowed_models: Option<AllowlistPin>,
    pub default_model: Option<String>,
    pub ask_user_question: Option<bool>,
    pub image_edit: Option<bool>,
    pub image_gen: Option<bool>,
    pub lsp_tools: Option<bool>,
    pub video_gen: Option<bool>,
    pub web_fetch: Option<bool>,
    pub write_file: Option<bool>,
}

impl RequirementsToml {
    #[must_use]
    pub fn from_value(layer: &toml::Value) -> Self {
        Self::deserialize(layer.clone()).unwrap_or_default()
    }

    pub fn tool_pins(&self) -> impl Iterator<Item = (ToolFeature, bool)> + '_ {
        ToolFeature::iter().filter_map(|tool| tool.pin(self).map(|val| (tool, val)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sourced<T> {
    pub value: T,
    pub source: RequirementsSource,
}

impl<T> Sourced<T> {
    #[must_use]
    pub fn new(value: T, source: RequirementsSource) -> Self {
        Self { value, source }
    }
}

impl<T> Deref for Sourced<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequirementsWithSources {
    pub allowed_models: Option<Sourced<AllowlistPin>>,
    pub default_model: Option<Sourced<String>>,
    pub ask_user_question: Option<Sourced<bool>>,
    pub image_edit: Option<Sourced<bool>>,
    pub image_gen: Option<Sourced<bool>>,
    pub lsp_tools: Option<Sourced<bool>>,
    pub video_gen: Option<Sourced<bool>>,
    pub web_fetch: Option<Sourced<bool>>,
    pub write_file: Option<Sourced<bool>>,
}

impl RequirementsWithSources {
    pub fn merge_unset_fields(&mut self, source: RequirementsSource, other: RequirementsToml) {
        macro_rules! fill_missing_take {
            ($base:expr, $other:expr, $source:expr, { $($field:ident),+ $(,)? }) => {
                $(if $base.$field.is_none()
                    && let Some(value) = $other.$field.take()
                {
                    $base.$field = Some(Sourced::new(value, $source.clone()));
                })+
            };
        }

        // A new RequirementsToml field fails to compile until this merge fills it
        let RequirementsToml {
            allowed_models: _,
            default_model: _,
            ask_user_question: _,
            image_edit: _,
            image_gen: _,
            lsp_tools: _,
            video_gen: _,
            web_fetch: _,
            write_file: _,
        } = &other;
        let mut other = other;
        fill_missing_take!(self, other, source, {
            allowed_models,
            default_model,
            ask_user_question,
            image_edit,
            image_gen,
            lsp_tools,
            video_gen,
            web_fetch,
            write_file
        });
    }
}

// An Err from de_allowlist drops the rest of the models section
fn de_allowlist<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<AllowlistPin>, D::Error> {
    let value = toml::Value::deserialize(deserializer)?;
    let Some(entries) = value.as_array() else {
        tracing::error!(
            section = "models",
            key = "allowed_models",
            kind = value.type_str(),
            "requirements value is not an array; the constraint fail-closes"
        );
        return Ok(Some(AllowlistPin::FailClosed));
    };
    let mut patterns = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(pattern) = entry.as_str() else {
            tracing::error!(
                section = "models",
                key = "allowed_models",
                kind = entry.type_str(),
                "requirements array entry is not a string; the constraint fail-closes"
            );
            return Ok(Some(AllowlistPin::FailClosed));
        };
        patterns.push(pattern.to_owned());
    }
    Ok(Some(AllowlistPin::List(patterns)))
}

fn de_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value.as_str().map(str::to_owned))
}

fn de_section<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

macro_rules! de_pin {
    ($name:ident, $key:literal) => {
        fn $name<'de, D: serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<bool>, D::Error> {
            let value = toml::Value::deserialize(deserializer)?;
            match value.as_bool() {
                Some(pin) => Ok(Some(pin)),
                None => {
                    tracing::error!(
                        section = "features",
                        key = $key,
                        kind = value.type_str(),
                        "requirements pin is not a boolean; the pin is ignored until the next \
                         launch, which will refuse to start"
                    );
                    Ok(None)
                }
            }
        }
    };
}

de_pin!(de_ask_user_question, "ask_user_question");
de_pin!(de_image_edit, "image_edit");
de_pin!(de_image_gen, "image_gen");
de_pin!(de_lsp_tools, "lsp_tools");
de_pin!(de_video_gen, "video_gen");
de_pin!(de_web_fetch, "web_fetch");
de_pin!(de_write_file, "write_file");

#[derive(Deserialize, Default)]
struct ModelsToml {
    #[serde(default, deserialize_with = "de_allowlist")]
    allowed_models: Option<AllowlistPin>,
    #[serde(default, rename = "default", deserialize_with = "de_string")]
    default_model: Option<String>,
}

#[derive(Deserialize, Default)]
struct FeaturesToml {
    #[serde(default, deserialize_with = "de_ask_user_question")]
    ask_user_question: Option<bool>,
    #[serde(default, deserialize_with = "de_image_edit")]
    image_edit: Option<bool>,
    #[serde(default, deserialize_with = "de_image_gen")]
    image_gen: Option<bool>,
    #[serde(default, deserialize_with = "de_lsp_tools")]
    lsp_tools: Option<bool>,
    #[serde(default, deserialize_with = "de_video_gen")]
    video_gen: Option<bool>,
    #[serde(default, deserialize_with = "de_web_fetch")]
    web_fetch: Option<bool>,
    #[serde(default, deserialize_with = "de_write_file")]
    write_file: Option<bool>,
}

#[derive(Deserialize, Default)]
struct RequirementsFile {
    #[serde(default, deserialize_with = "de_section")]
    models: ModelsToml,
    #[serde(default, deserialize_with = "de_section")]
    features: FeaturesToml,
}

impl From<RequirementsFile> for RequirementsToml {
    fn from(file: RequirementsFile) -> Self {
        Self {
            allowed_models: file.models.allowed_models,
            default_model: file.models.default_model,
            ask_user_question: file.features.ask_user_question,
            image_edit: file.features.image_edit,
            image_gen: file.features.image_gen,
            lsp_tools: file.features.lsp_tools,
            video_gen: file.features.video_gen,
            web_fetch: file.features.web_fetch,
            write_file: file.features.write_file,
        }
    }
}

#[cfg(test)]
#[path = "config_requirements_tests.rs"]
mod tests;

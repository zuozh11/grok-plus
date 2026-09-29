use std::path::PathBuf;

use super::*;

fn user() -> RequirementsSource {
    RequirementsSource::File(PathBuf::from("/home/dev/.grok/requirements.toml"))
}

fn system() -> RequirementsSource {
    RequirementsSource::File(PathBuf::from("/etc/grok/requirements.toml"))
}

fn parse(text: &str) -> RequirementsToml {
    toml::from_str(text).expect("requirements text is valid TOML")
}

fn merge(layers: &[(&RequirementsSource, &str)]) -> RequirementsWithSources {
    let mut merged = RequirementsWithSources::default();
    for (source, text) in layers {
        merged.merge_unset_fields((*source).clone(), parse(text));
    }
    merged
}

fn list(patterns: &[&str]) -> AllowlistPin {
    AllowlistPin::List(
        patterns
            .iter()
            .map(|pattern| (*pattern).to_owned())
            .collect(),
    )
}

#[test]
fn merge_copies_every_field_and_sets_its_source() {
    let source = user();
    let merged = merge(&[(
        &source,
        "\
[models]
allowed_models = [\"grok-4*\"]
default = \"grok-4\"

[features]
ask_user_question = false
image_edit = true
image_gen = false
lsp_tools = true
video_gen = false
web_fetch = true
write_file = false
",
    )]);

    assert_eq!(
        Some(Sourced::new(list(&["grok-4*"]), source.clone())),
        merged.allowed_models
    );
    assert_eq!(
        Some(Sourced::new("grok-4".to_owned(), source.clone())),
        merged.default_model
    );
    assert_eq!(
        Some(Sourced::new(false, source.clone())),
        merged.ask_user_question
    );
    assert_eq!(Some(Sourced::new(true, source.clone())), merged.image_edit);
    assert_eq!(Some(Sourced::new(false, source.clone())), merged.image_gen);
    assert_eq!(Some(Sourced::new(true, source.clone())), merged.lsp_tools);
    assert_eq!(Some(Sourced::new(false, source.clone())), merged.video_gen);
    assert_eq!(Some(Sourced::new(true, source.clone())), merged.web_fetch);
    assert_eq!(Some(Sourced::new(false, source)), merged.write_file);
}

#[test]
fn first_writer_wins() {
    let high = system();
    let low = user();
    let cases = [
        (
            vec![
                (&high, "[models]\nallowed_models = [\"grok-4\"]\n"),
                (&low, "[models]\nallowed_models = [\"grok-3\"]\n"),
            ],
            Some(Sourced::new(list(&["grok-4"]), high.clone())),
            None,
        ),
        (
            vec![
                (&high, "[models]\ndefault = \"grok-4\"\n"),
                (&low, "[models]\ndefault = \"grok-3\"\n"),
            ],
            None,
            Some(Sourced::new("grok-4".to_owned(), high.clone())),
        ),
        (
            vec![
                (&high, "[models]\ndefault = \"grok-4\"\n"),
                (&low, "[models]\nallowed_models = [\"grok-3\"]\n"),
            ],
            Some(Sourced::new(list(&["grok-3"]), low.clone())),
            Some(Sourced::new("grok-4".to_owned(), high.clone())),
        ),
    ];

    for (layers, allowed_models, default_model) in cases {
        let merged = merge(&layers);
        assert_eq!(allowed_models, merged.allowed_models, "{layers:?}");
        assert_eq!(default_model, merged.default_model, "{layers:?}");
    }

    let merged = merge(&[
        (&high, "[features]\nimage_gen = true\n"),
        (&low, "[features]\nimage_gen = false\nwrite_file = false\n"),
    ]);
    assert_eq!(Some(Sourced::new(true, high)), merged.image_gen);
    assert_eq!(Some(Sourced::new(false, low)), merged.write_file);
}

#[test]
fn malformed_allowed_models_fail_closed() {
    let cases = [
        (
            "[models]\nallowed_models = \"grok-4\"\n",
            AllowlistPin::FailClosed,
        ),
        (
            "[models]\nallowed_models = [\"grok-4\", 4]\n",
            AllowlistPin::FailClosed,
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(Some(expected), parse(text).allowed_models, "{text:?}");
    }

    let text =
        "[models]\ndefault = 4\nallowed_models = [\"grok-4\"]\n[features]\nimage_gen = false\n";
    let parsed = parse(text);
    assert_eq!(None, parsed.default_model);
    assert_eq!(Some(list(&["grok-4"])), parsed.allowed_models);
    assert_eq!(Some(false), parsed.image_gen);

    let source = RequirementsSource::Mdm;
    let merged = merge(&[(&source, "[models]\nallowed_models = 4\n")]);
    assert_eq!(
        Some(Sourced::new(AllowlistPin::FailClosed, source)),
        merged.allowed_models
    );
}

#[test]
fn tool_pins_keep_booleans_and_skip_other_types() {
    let parsed = parse(
        "\
[features]
image_gen = false
web_fetch = true
video_gen = \"off\"
voice_mode = false
",
    );
    assert_eq!(Some(false), parsed.image_gen);
    assert_eq!(Some(true), parsed.web_fetch);
    assert_eq!(None, parsed.video_gen);
    assert_eq!(None, parsed.ask_user_question);
    assert_eq!(
        vec![
            (ToolFeature::ImageGen, false),
            (ToolFeature::WebFetch, true)
        ],
        parsed.tool_pins().collect::<Vec<_>>()
    );
}

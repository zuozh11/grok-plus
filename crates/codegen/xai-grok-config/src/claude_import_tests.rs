use super::*;

#[test]
fn only_an_explicit_true_marker_reads_as_imported() {
    let cases = [
        (None, false),
        (Some("[other]\nkey = \"value\"\n"), false),
        (Some("[claude_compat]\nimported = false\n"), false),
        (Some("[claude_compat]\nimported = "), false),
        (Some("[claude_compat]\nimported = true\n"), true),
    ];
    for (config, expected) in cases {
        let home = tempfile::tempdir().unwrap();
        if let Some(config) = config {
            std::fs::write(home.path().join("config.toml"), config).unwrap();
        }
        assert_eq!(expected, is_claude_import_marked(home.path()), "{config:?}");
    }
}

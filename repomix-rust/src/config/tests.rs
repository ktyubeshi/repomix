// repomix-rust/src/config/tests.rs
#[cfg(test)]
mod tests {
    use crate::cli::Cli;
    use crate::config::schema::{
        default_file_path_map, OutputConfig, RepomixConfig, RepomixOutputStyle,
        TokenCountTreeConfig,
    };
    use clap::Parser;
    use proptest::prelude::*;
    use serde_json;

    #[test]
    fn test_token_count_tree_config_deserialization() {
        // Test boolean true
        let json_true = r#"true"#;
        let config_true: TokenCountTreeConfig = serde_json::from_str(json_true).unwrap();
        assert_eq!(config_true, TokenCountTreeConfig::Bool(true));

        // Test boolean false
        let json_false = r#"false"#;
        let config_false: TokenCountTreeConfig = serde_json::from_str(json_false).unwrap();
        assert_eq!(config_false, TokenCountTreeConfig::Bool(false));

        // Test integer
        let json_int = r#"50000"#;
        let config_int: TokenCountTreeConfig = serde_json::from_str(json_int).unwrap();
        assert_eq!(config_int, TokenCountTreeConfig::Threshold(50000));
    }

    #[test]
    fn test_token_count_tree_config_deserialization_string() {
        let json_str = r#""100""#;
        let config_str: TokenCountTreeConfig = serde_json::from_str(json_str).unwrap();
        assert_eq!(config_str, TokenCountTreeConfig::Text("100".to_string()));
    }

    #[test]
    fn test_token_count_tree_config_rejects_invalid_types() {
        let json_invalid = r#"[]"#;
        let result: Result<TokenCountTreeConfig, _> = serde_json::from_str(json_invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_output_config_deserialization_with_token_count_tree() {
        let json = r#"{
            "tokenCountTree": 50000
        }"#;
        let config: OutputConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.token_count_tree,
            TokenCountTreeConfig::Threshold(50000)
        );
    }

    #[test]
    fn test_repomix_config_merge_no_flags() {
        let mut config = RepomixConfig::default();
        // Initially true by default
        config.output.file_summary = true;

        // Simulate --no-file-summary
        let cli = Cli::parse_from(&["repomix", "--no-file-summary"]);

        // The parse_from above actually sets the flag!
        // Since we changed `no-*` flags to be bool and SetTrue, presence means true.
        assert_eq!(cli.no_file_summary, true);

        config = config.merge_with_cli(&cli);

        assert_eq!(config.output.file_summary, false);
    }

    fn output_style_strategy() -> impl Strategy<Value = RepomixOutputStyle> {
        prop_oneof![
            Just(RepomixOutputStyle::Xml),
            Just(RepomixOutputStyle::Markdown),
            Just(RepomixOutputStyle::Json),
            Just(RepomixOutputStyle::Plain),
        ]
    }

    proptest! {
        #[test]
        fn token_count_tree_config_roundtrips_bools(value in any::<bool>()) {
            let json_value = serde_json::to_string(&value).unwrap();
            let parsed: TokenCountTreeConfig = serde_json::from_str(&json_value).unwrap();
            prop_assert_eq!(parsed, TokenCountTreeConfig::Bool(value));
        }

        #[test]
        fn token_count_tree_config_roundtrips_numbers(value in any::<u64>()) {
            let json_value = serde_json::to_string(&value).unwrap();
            let parsed: TokenCountTreeConfig = serde_json::from_str(&json_value).unwrap();
            prop_assert_eq!(parsed, TokenCountTreeConfig::Threshold(value));
        }

        #[test]
        fn merge_with_cli_respects_no_flags(
            file_summary in any::<bool>(),
            directory_structure in any::<bool>(),
            copy_to_clipboard in any::<bool>(),
            // token_count_tree removed from here, tested separately
        ) {
            fn flag_args(flag: &str, no_flag: &str, enabled: bool) -> [String; 1] {
                if enabled {
                    [format!("--{}", flag)]
                } else {
                    [format!("--{}", no_flag)]
                }
            }

            let mut args = vec!["repomix".to_string()];
            args.extend(flag_args("file-summary", "no-file-summary", file_summary));
            args.extend(flag_args(
                "directory-structure",
                "no-directory-structure",
                directory_structure,
            ));
            args.extend(flag_args("copy", "no-copy", copy_to_clipboard));

            let cli = Cli::parse_from(args);
            let merged = RepomixConfig::default().merge_with_cli(&cli);

            prop_assert_eq!(merged.output.file_summary, file_summary);
            prop_assert_eq!(merged.output.directory_structure, directory_structure);
            prop_assert_eq!(merged.output.copy_to_clipboard, copy_to_clipboard);
        }

        #[test]
        fn merge_with_cli_token_count_tree(
            enabled in any::<bool>(),
            threshold in any::<Option<u64>>(),
        ) {
            let mut args = vec!["repomix".to_string()];
            if !enabled {
                args.push("--no-token-count-tree".to_string());
            } else {
                if let Some(val) = threshold {
                    args.push(format!("--token-count-tree={}", val));
                } else {
                    args.push("--token-count-tree".to_string());
                }
            }

            let cli = Cli::parse_from(args);
            let merged = RepomixConfig::default().merge_with_cli(&cli);

            if !enabled {
                prop_assert_eq!(merged.output.token_count_tree, TokenCountTreeConfig::Bool(false));
            } else {
                if let Some(val) = threshold {
                    prop_assert_eq!(merged.output.token_count_tree, TokenCountTreeConfig::Threshold(val));
                } else {
                    // Default missing value is "true" -> Bool(true)
                    prop_assert_eq!(merged.output.token_count_tree, TokenCountTreeConfig::Bool(true));
                }
            }
        }

        #[test]
        fn merge_with_cli_sets_style_default_path_when_not_explicit(style in output_style_strategy()) {
            let mut config = RepomixConfig::default();
            config.output.style = style.clone();
            config.output.file_path_explicit = false;

            let cli = Cli::parse_from(&["repomix"]);
            let merged = config.merge_with_cli(&cli);

            let expected_path = default_file_path_map()
                .get(&style)
                .expect("default path should exist for all styles")
                .clone();

            prop_assert_eq!(merged.output.file_path, Some(expected_path));
        }

        #[test]
        fn merge_with_cli_uses_cli_style_default_path_without_file_path(cli_style in output_style_strategy()) {
            let cli = Cli::parse_from(&["repomix", &format!("--style={}", cli_style.to_string())]);
            let merged = RepomixConfig::default().merge_with_cli(&cli);

            let expected_path = default_file_path_map()
                .get(&cli_style)
                .expect("default path should exist for all styles")
                .clone();

            prop_assert_eq!(merged.output.file_path, Some(expected_path));
        }

        #[test]
        fn merge_with_cli_keeps_explicit_file_path(
            cli_style in output_style_strategy(),
            path in proptest::string::string_regex("[A-Za-z0-9_/]{5,20}")
                .unwrap()
                .prop_map(|s| format!("{s}.txt"))
        ) {
            let mut config = RepomixConfig::default();
            config.output.file_path = Some(path.clone());
            config.output.file_path_explicit = true;

            let cli = Cli::parse_from(&["repomix", &format!("--style={}", cli_style.to_string())]);
            let merged = config.merge_with_cli(&cli);

            prop_assert_eq!(merged.output.file_path, Some(path));
        }
    }
}

//! Tests for upgrade rules (Go 1.22/1.23/1.24/1.25)

use crate::rules::{upgrade::*, Rule, RuleCategory};
use crate::Severity;

#[test]
fn test_up1221_integer_range_metadata() {
    let rule = Up1221IntegerRange;
    let meta = rule.metadata();

    assert_eq!(meta.code, "UP1221");
    assert_eq!(meta.name, "use-integer-range");
    assert_eq!(meta.category, RuleCategory::Upgrade);
    assert_eq!(meta.min_go_version, Some("1.22"));
    assert_eq!(meta.default_severity, Severity::Info);
}

#[test]
fn test_up1222_math_rand_v2_metadata() {
    let rule = Up1222MathRandV2;
    let meta = rule.metadata();

    assert_eq!(meta.code, "UP1222");
    assert_eq!(meta.min_go_version, Some("1.22"));
}

#[test]
fn test_up1231_slices_iter_metadata() {
    let rule = Up1231SlicesIter;
    let meta = rule.metadata();

    assert_eq!(meta.code, "UP1231");
    assert_eq!(meta.min_go_version, Some("1.23"));
}

#[test]
fn test_up1241_rand_text_metadata() {
    let rule = Up1241RandText;
    let meta = rule.metadata();

    assert_eq!(meta.code, "UP1241");
    assert_eq!(meta.min_go_version, Some("1.24"));
}

#[test]
fn test_up1251_json_v2_metadata() {
    let rule = Up1251JsonV2;
    let meta = rule.metadata();

    assert_eq!(meta.code, "UP1251");
    assert_eq!(meta.min_go_version, Some("1.25"));
}

#[test]
fn test_all_upgrade_rules_have_version() {
    let rules = get_upgrade_rules();

    for rule in &rules {
        let meta = rule.metadata();
        assert!(
            meta.min_go_version.is_some(),
            "Rule {} should have min_go_version",
            meta.code
        );

        // Verify version format (should be "1.XX")
        let version = meta.min_go_version.unwrap();
        assert!(
            version.starts_with("1."),
            "Version should start with 1.: {}",
            version
        );

        // Verify version is 1.22, 1.23, 1.24, or 1.25
        let minor: u32 = version[2..].parse().expect("Valid minor version");
        assert!(
            (22..=25).contains(&minor),
            "Version should be 1.22-1.25: {}",
            version
        );
    }
}

#[test]
fn test_upgrade_rules_count() {
    let rules = get_upgrade_rules();

    // Should have 18 rules total (5 + 5 + 4 + 4)
    assert_eq!(rules.len(), 18, "Expected 18 upgrade rules");

    // Count by version
    let v22 = rules
        .iter()
        .filter(|r| r.metadata().min_go_version == Some("1.22"))
        .count();
    let v23 = rules
        .iter()
        .filter(|r| r.metadata().min_go_version == Some("1.23"))
        .count();
    let v24 = rules
        .iter()
        .filter(|r| r.metadata().min_go_version == Some("1.24"))
        .count();
    let v25 = rules
        .iter()
        .filter(|r| r.metadata().min_go_version == Some("1.25"))
        .count();

    assert_eq!(v22, 5, "Expected 5 Go 1.22 rules");
    assert_eq!(v23, 5, "Expected 5 Go 1.23 rules");
    assert_eq!(v24, 4, "Expected 4 Go 1.24 rules");
    assert_eq!(v25, 4, "Expected 4 Go 1.25 rules");
}

#[test]
fn test_version_parsing() {
    use crate::config::Config;
    use crate::rules::{get_enabled_rules, parse_version};

    // Test version parsing
    assert_eq!(parse_version("1.21"), Some(1021));
    assert_eq!(parse_version("1.22"), Some(1022));
    assert_eq!(parse_version("1.25"), Some(1025));
    assert_eq!(parse_version("2.0"), Some(2000));
    assert_eq!(parse_version("invalid"), None);

    // Test with config
    let mut config = Config::default();
    config.global.target_go_version = "1.21".to_string();

    let rules_121 = get_enabled_rules(&config);
    let upgrade_count_121 = rules_121
        .iter()
        .filter(|r| {
            r.metadata().category == RuleCategory::Upgrade && r.metadata().min_go_version.is_some()
        })
        .count();
    assert_eq!(
        upgrade_count_121, 0,
        "Go 1.21 should have 0 versioned upgrade rules enabled"
    );

    // Test Go 1.22
    config.global.target_go_version = "1.22".to_string();
    let rules_122 = get_enabled_rules(&config);
    let upgrade_count_122 = rules_122
        .iter()
        .filter(|r| {
            r.metadata().category == RuleCategory::Upgrade && r.metadata().min_go_version.is_some()
        })
        .count();
    assert_eq!(
        upgrade_count_122, 5,
        "Go 1.22 should have 5 versioned upgrade rules enabled"
    );

    // Test Go 1.25
    config.global.target_go_version = "1.25".to_string();
    let rules_125 = get_enabled_rules(&config);
    let upgrade_count_125 = rules_125
        .iter()
        .filter(|r| {
            r.metadata().category == RuleCategory::Upgrade && r.metadata().min_go_version.is_some()
        })
        .count();
    assert_eq!(
        upgrade_count_125, 18,
        "Go 1.25 should have all 18 versioned upgrade rules enabled"
    );
}

use memorable_tui::config::{normalize_url, Config, DEFAULT_HOST, PRESET_HOST_LOCAL};

#[test]
fn test_normalize_url() {
    assert_eq!(
        normalize_url("http://example.com:3011/"),
        "http://example.com:3011"
    );
    assert_eq!(
        normalize_url("http://localhost:3000/"),
        "http://localhost:3000"
    );
    assert_eq!(
        normalize_url("example.com:3011"),
        "http://example.com:3011"
    );
    assert_eq!(
        normalize_url("http://localhost:3000/api/v1"),
        "http://localhost:3000"
    );
    assert_eq!(
        normalize_url("http://localhost:3000/api/v1/"),
        "http://localhost:3000"
    );
    assert_eq!(
        normalize_url(""),
        DEFAULT_HOST
    );
}

#[test]
fn test_default_without_env_or_flags() {
    let args = vec!["memorable-tui".to_string()];
    let cfg = Config::parse_from_args_and_env(&args, None);
    assert_eq!(cfg.base_url, "http://localhost:3000");
    assert!(cfg.allow_host_switcher);
}

#[test]
fn test_default_with_memorable_remote_url() {
    let args = vec!["memorable-tui".to_string()];
    let cfg = Config::parse_from_args_and_env(&args, Some("http://remote-host:3011"));
    assert_eq!(cfg.base_url, "http://remote-host:3011");
    // Locked by default when remote url is in environment
    assert!(!cfg.allow_host_switcher);
}

#[test]
fn test_allow_switch_override_with_remote_url() {
    let args = vec![
        "memorable-tui".to_string(),
        "--allow-switch".to_string(),
    ];
    let cfg = Config::parse_from_args_and_env(&args, Some("http://remote-host:3011"));
    assert_eq!(cfg.base_url, "http://remote-host:3011");
    assert!(cfg.allow_host_switcher);
}

#[test]
fn test_local_flag_overrides_remote_env() {
    let args = vec!["memorable-tui".to_string(), "--local".to_string()];
    let cfg = Config::parse_from_args_and_env(&args, Some("http://remote-host:3011"));
    assert_eq!(cfg.base_url, PRESET_HOST_LOCAL);
}

#[test]
fn test_custom_url_flag_overrides_remote_env() {
    let args = vec![
        "memorable-tui".to_string(),
        "--url".to_string(),
        "http://custom:9999".to_string(),
    ];
    let cfg = Config::parse_from_args_and_env(&args, Some("http://remote-host:3011"));
    assert_eq!(cfg.base_url, "http://custom:9999");
}

use memorable_tui::app::App;
use memorable_tui::config::Config;

#[test]
fn test_live_connection_to_memorable() {
    let url_opt = std::env::var("MEMORABLE_REMOTE_URL").or_else(|_| std::env::var("MEMORABLE_URL")).ok();

    let url = match url_opt {
        Some(u) => u,
        None => {
            println!("Skipping test_live_connection_to_memorable: MEMORABLE_REMOTE_URL is not set.");
            return;
        }
    };

    let config = Config::new(&url);
    let app = App::new(config);

    assert!(app.stats.total_memories >= 1, "Expected total_memories >= 1");
    assert!(!app.memories.is_empty(), "Expected memories list to not be empty");

    let current = app.current_memory.as_ref().expect("Expected current_memory to be loaded");
    assert_eq!(current.id, 5);
    assert_eq!(current.title, "Writing Rules");
    assert!(current.body.contains("Before writing anything"));
}

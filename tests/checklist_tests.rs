use memorable_tui::checklist::{
    append_to_memory, checklist_progress, normalize_task_line, parse_checklist_items,
    toggle_checklist_item_by_index,
};

#[test]
fn test_parse_and_progress() {
    let md = r#"# Architecture Notes
- [ ] Review ADR 001
- [x] Benchmark pgvector HNSW
* [ ] Finalize API schema
Some regular markdown text
  - [X] Nested item
"#;
    let items = parse_checklist_items(md);
    assert_eq!(items.len(), 4);
    assert_eq!(items[0].text, "Review ADR 001");
    assert!(!items[0].completed);
    assert_eq!(items[1].text, "Benchmark pgvector HNSW");
    assert!(items[1].completed);
    assert_eq!(items[2].text, "Finalize API schema");
    assert_eq!(items[3].indent, "  ");
    assert!(items[3].completed);

    let (done, total) = checklist_progress(md);
    assert_eq!(done, 2);
    assert_eq!(total, 4);
}

#[test]
fn test_toggle_item() {
    let md = "- [ ] Item A\n- [x] Item B\n";
    let (updated, item) = toggle_checklist_item_by_index(md, 0).unwrap();
    assert!(item.completed);
    assert_eq!(updated, "- [x] Item A\n- [x] Item B\n");

    let (toggled_back, item2) = toggle_checklist_item_by_index(&updated, 0).unwrap();
    assert!(!item2.completed);
    assert_eq!(toggled_back, "- [ ] Item A\n- [x] Item B\n");
}

#[test]
fn test_normalize_task() {
    assert_eq!(normalize_task_line("Run tests"), "- [ ] Run tests");
    assert_eq!(normalize_task_line("[ ] Run tests"), "- [ ] Run tests");
    assert_eq!(normalize_task_line("- [x] Run tests"), "- [x] Run tests");
}

#[test]
fn test_append_prose_and_task() {
    let base = "# Memory 5\n\nInitial context.\n";
    let res = append_to_memory(base, "Vector dimensions are 1024.", Some("Vector Config"), false);
    assert_eq!(
        res,
        "# Memory 5\n\nInitial context.\n\n## Vector Config\nVector dimensions are 1024.\n"
    );

    let res_task = append_to_memory(&res, "Index embedding column", Some("Action Items"), true);
    assert!(res_task.contains("## Action Items\n- [ ] Index embedding column\n"));
}

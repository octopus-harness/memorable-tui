use memorable_tui::models::{
    MemoryDetail, MemoryListResponse, NeighborApiResponse, SearchResponse, Stats,
};

#[test]
fn test_deserialize_memory_list() {
    let json_data = r##"{
        "memories": [
            {
                "id": 5,
                "title": "Writing Rules",
                "pinned": true,
                "pinned_position": null,
                "attachments_count": 0,
                "embedded": true,
                "embedding_model": "qwen3-embedding:0.6b",
                "embedded_at": "2026-11-17T02:14:16Z",
                "created_at": "2026-11-16T16:45:56Z",
                "updated_at": "2026-11-17T02:14:16Z",
                "body": "Before writing anything..."
            }
        ],
        "meta": {
            "limit": 50,
            "offset": 0,
            "total_count": 1
        }
    }"##;

    let res: MemoryListResponse = serde_json::from_str(json_data).unwrap();
    assert_eq!(res.memories.len(), 1);
    let mem = &res.memories[0];
    assert_eq!(mem.id, 5);
    assert_eq!(mem.title, "Writing Rules");
    assert!(mem.pinned);
    assert!(mem.embedded);
    assert_eq!(mem.embedding_model.as_deref(), Some("qwen3-embedding:0.6b"));
}

#[test]
fn test_deserialize_memory_detail() {
    let json_data = r##"{
        "id": 5,
        "title": "Writing Rules",
        "pinned": true,
        "pinned_position": null,
        "attachments_count": 1,
        "embedded": true,
        "embedding_model": "qwen3-embedding:0.6b",
        "embedded_at": "2026-11-17T02:14:16Z",
        "created_at": "2026-11-16T16:45:56Z",
        "updated_at": "2026-11-17T02:14:16Z",
        "body": "# Writing Rules\n- [ ] Follow voice guidelines",
        "attachments": [
            {
                "id": 101,
                "filename": "guidelines.pdf",
                "byte_size": 2048,
                "content_type": "application/pdf",
                "url": "/rails/active_storage/blobs/redirect/ey.../guidelines.pdf",
                "created_at": "2026-11-17T02:15:00Z"
            }
        ]
    }"##;

    let detail: MemoryDetail = serde_json::from_str(json_data).unwrap();
    assert_eq!(detail.id, 5);
    assert_eq!(detail.attachments.len(), 1);
    assert_eq!(detail.attachments[0].filename, "guidelines.pdf");
    assert_eq!(detail.attachments[0].byte_size, Some(2048));
}

#[test]
fn test_deserialize_stats() {
    let json_data = r#"{"total_memories":12,"pinned_memories":3,"total_attachments":5}"#;
    let stats: Stats = serde_json::from_str(json_data).unwrap();
    assert_eq!(stats.total_memories, 12);
    assert_eq!(stats.pinned_memories, 3);
    assert_eq!(stats.total_attachments, 5);
}

#[test]
fn test_deserialize_neighbors() {
    let json_data = r#"{
        "status": "ok",
        "memory": {
            "id": 5,
            "title": "Writing Rules",
            "body": "rules...",
            "pinned": false,
            "pinned_position": null,
            "attachments_count": 0,
            "embedded": true,
            "embedding_model": "qwen3-embedding:0.6b",
            "embedded_at": "2026-11-17T02:14:16Z",
            "created_at": "2026-11-16T16:45:56Z",
            "updated_at": "2026-11-17T02:14:16Z"
        },
        "neighbors": [
            {
                "score": 0.895123,
                "memory": {
                    "id": 8,
                    "title": "Voice Guidelines",
                    "body": "grounded sober...",
                    "pinned": false,
                    "pinned_position": null,
                    "attachments_count": 0,
                    "embedded": true,
                    "embedding_model": "qwen3-embedding:0.6b",
                    "embedded_at": "2026-11-17T03:00:00Z",
                    "created_at": "2026-11-17T03:00:00Z",
                    "updated_at": "2026-11-17T03:00:00Z"
                }
            }
        ]
    }"#;

    let res: NeighborApiResponse = serde_json::from_str(json_data).unwrap();
    assert_eq!(res.status, "ok");
    assert_eq!(res.neighbors.len(), 1);
    assert_eq!(res.neighbors[0].score, Some(0.895123));
    assert_eq!(res.neighbors[0].memory.title, "Voice Guidelines");
}

#[test]
fn test_deserialize_search() {
    let json_data = r#"{
        "status": "ok",
        "results": [
            {
                "type": "Memory",
                "score": 0.680946,
                "data": {
                    "id": 5,
                    "title": "Writing Rules",
                    "body": "rules...",
                    "pinned": true,
                    "pinned_position": null,
                    "attachments_count": 0,
                    "embedded": true,
                    "embedding_model": "qwen3-embedding:0.6b",
                    "embedded_at": "2026-11-17T02:14:16Z",
                    "created_at": "2026-11-16T16:45:56Z",
                    "updated_at": "2026-11-17T02:14:16Z"
                }
            }
        ],
        "meta": {
            "mode": "semantic",
            "query": "Rules",
            "count": 1
        }
    }"#;

    let res: SearchResponse = serde_json::from_str(json_data).unwrap();
    assert_eq!(res.status, "ok");
    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].score, Some(0.680946));
    assert_eq!(res.results[0].data.title, "Writing Rules");
}

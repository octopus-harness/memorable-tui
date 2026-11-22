use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    All,
    Pinned,
    Embedded,
    Unembedded,
    Recent,
}

impl Scope {
    pub fn all() -> &'static [Scope] {
        &[
            Scope::All,
            Scope::Pinned,
            Scope::Embedded,
            Scope::Unembedded,
            Scope::Recent,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Scope::All => "All Memories",
            Scope::Pinned => "★ Pinned",
            Scope::Embedded => "Vector Embedded",
            Scope::Unembedded => "Unembedded",
            Scope::Recent => "Recent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub pinned: bool,
    pub pinned_position: Option<i64>,
    #[serde(default)]
    pub attachments_count: i64,
    #[serde(default)]
    pub embedded: bool,
    pub embedding_model: Option<String>,
    pub embedded_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDetail {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub pinned: bool,
    pub pinned_position: Option<i64>,
    #[serde(default)]
    pub attachments_count: i64,
    #[serde(default)]
    pub embedded: bool,
    pub embedding_model: Option<String>,
    pub embedded_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub rendered_html: Option<String>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: i64,
    pub filename: String,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub created_at: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborResult {
    pub score: Option<f64>,
    pub memory: Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    #[serde(rename = "type")]
    pub hit_type: Option<String>,
    pub score: Option<f64>,
    pub data: Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub status: String,
    #[serde(default)]
    pub results: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborApiResponse {
    pub status: String,
    pub memory: Option<Memory>,
    #[serde(default)]
    pub neighbors: Vec<NeighborResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryListResponse {
    #[serde(default)]
    pub memories: Vec<Memory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentListResponse {
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    #[serde(default)]
    pub total_memories: i64,
    #[serde(default)]
    pub pinned_memories: i64,
    #[serde(default)]
    pub total_attachments: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInput {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

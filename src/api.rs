use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::blocking::multipart;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

use crate::config::Config;
use crate::models::{
    Attachment, AttachmentListResponse, Memory, MemoryDetail, MemoryInput, MemoryListResponse,
    NeighborApiResponse, NeighborResult, SearchResponse, Stats,
};

#[derive(Debug, Clone)]
pub struct ApiClient {
    pub base_url: String,
    pub api_token: Option<String>,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(config: &Config) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            base_url: config.base_url.clone(),
            api_token: config.api_token.clone(),
            client,
        }
    }

    pub fn set_base_url(&mut self, new_url: &str) {
        self.base_url = new_url.to_string();
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(token) = &self.api_token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }

    fn check_response(&self, res: reqwest::blocking::Response) -> Result<String> {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        if !status.is_success() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                    bail!("API Error ({}): {}", status, err);
                }
            }
            bail!("API Request failed ({}): {}", status, body);
        }
        Ok(body)
    }

    pub fn list_memories(
        &self,
        pinned: Option<bool>,
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        let mut url = format!("{}/api/v1/memories?limit={}", self.base_url, limit);
        if let Some(p) = pinned {
            url.push_str(&format!("&pinned={}", p));
        }
        if let Some(q) = query.filter(|s| !s.trim().is_empty()) {
            url.push_str(&format!("&q={}", urlencoding::encode(q.trim())));
        }

        let res = self.client.get(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let list: MemoryListResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse memories list: {}", body))?;
        Ok(list.memories)
    }

    pub fn get_memory(&self, id: i64, render_markdown: bool) -> Result<MemoryDetail> {
        let url = format!(
            "{}/api/v1/memories/{}?render_markdown={}",
            self.base_url, id, render_markdown
        );
        let res = self.client.get(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let detail: MemoryDetail = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse memory detail: {}", body))?;
        Ok(detail)
    }

    pub fn create_memory(&self, input: &MemoryInput) -> Result<MemoryDetail> {
        let url = format!("{}/api/v1/memories", self.base_url);
        let payload = serde_json::json!({ "memory": input });
        let res = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&payload)
            .send()?;
        let body = self.check_response(res)?;
        let detail: MemoryDetail = serde_json::from_str(&body)?;
        Ok(detail)
    }

    pub fn update_memory(&self, id: i64, input: &MemoryInput) -> Result<MemoryDetail> {
        let url = format!("{}/api/v1/memories/{}", self.base_url, id);
        let payload = serde_json::json!({ "memory": input });
        let res = self
            .client
            .patch(&url)
            .headers(self.headers())
            .json(&payload)
            .send()?;
        let body = self.check_response(res)?;
        let detail: MemoryDetail = serde_json::from_str(&body)?;
        Ok(detail)
    }

    pub fn delete_memory(&self, id: i64) -> Result<()> {
        let url = format!("{}/api/v1/memories/{}", self.base_url, id);
        let res = self.client.delete(&url).headers(self.headers()).send()?;
        self.check_response(res)?;
        Ok(())
    }

    pub fn toggle_pin(&self, id: i64) -> Result<bool> {
        let url = format!("{}/api/v1/memories/{}/toggle_pin", self.base_url, id);
        let res = self.client.patch(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let val: serde_json::Value = serde_json::from_str(&body)?;
        let pinned = val.get("pinned").and_then(|v| v.as_bool()).unwrap_or(false);
        Ok(pinned)
    }

    pub fn reorder_pinned(&self, ids: &[i64]) -> Result<()> {
        let url = format!("{}/api/v1/memories/reorder_pinned", self.base_url);
        let payload = serde_json::json!({ "memory_ids": ids });
        let res = self
            .client
            .patch(&url)
            .headers(self.headers())
            .json(&payload)
            .send()?;
        self.check_response(res)?;
        Ok(())
    }

    pub fn get_neighbors(&self, id: i64, limit: usize) -> Result<Vec<NeighborResult>> {
        let url = format!(
            "{}/api/v1/memories/{}/neighbors?limit={}",
            self.base_url, id, limit
        );
        let res = self.client.get(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let resp: NeighborApiResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse neighbor response: {}", body))?;
        Ok(resp.neighbors)
    }

    pub fn generate_embedding(&self, id: i64) -> Result<()> {
        let url = format!("{}/api/v1/memories/{}/generate_embedding", self.base_url, id);
        let res = self.client.post(&url).headers(self.headers()).send()?;
        self.check_response(res)?;
        Ok(())
    }

    pub fn search(&self, query: &str, mode: &str, limit: usize) -> Result<Vec<NeighborResult>> {
        let url = format!(
            "{}/api/v1/search?q={}&mode={}&limit={}",
            self.base_url,
            urlencoding::encode(query.trim()),
            mode,
            limit
        );
        let res = self.client.get(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let resp: SearchResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse search response: {}", body))?;

        let results = resp
            .results
            .into_iter()
            .map(|hit| NeighborResult {
                score: hit.score,
                memory: hit.data,
            })
            .collect();
        Ok(results)
    }

    pub fn get_stats(&self) -> Result<Stats> {
        let url = format!("{}/api/v1/stats", self.base_url);
        let res = self.client.get(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let stats: Stats = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse stats: {}", body))?;
        Ok(stats)
    }

    pub fn list_attachments(&self, memory_id: i64) -> Result<Vec<Attachment>> {
        let url = format!("{}/api/v1/memories/{}/attachments", self.base_url, memory_id);
        let res = self.client.get(&url).headers(self.headers()).send()?;
        let body = self.check_response(res)?;
        let resp: AttachmentListResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse attachment list: {}", body))?;
        Ok(resp.attachments)
    }

    pub fn upload_attachment(&self, memory_id: i64, file_path: &Path) -> Result<()> {
        let url = format!("{}/api/v1/memories/{}/attachments", self.base_url, memory_id);
        let form = multipart::Form::new().file("files[]", file_path)?;

        let mut req = self.client.post(&url);
        if let Some(token) = &self.api_token {
            req = req.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        let res = req.multipart(form).send()?;
        self.check_response(res)?;
        Ok(())
    }

    pub fn delete_attachment(&self, memory_id: i64, attachment_id: i64) -> Result<()> {
        let url = format!(
            "{}/api/v1/memories/{}/attachments/{}",
            self.base_url, memory_id, attachment_id
        );
        let res = self.client.delete(&url).headers(self.headers()).send()?;
        self.check_response(res)?;
        Ok(())
    }

    pub fn download_attachment(&self, raw_url: &str, dest_path: &Path) -> Result<()> {
        let full_url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
            raw_url.to_string()
        } else {
            format!("{}{}", self.base_url, raw_url)
        };

        let mut req = self.client.get(&full_url);
        if let Some(token) = &self.api_token {
            req = req.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        let res = req.send()?;
        if !res.status().is_success() {
            bail!("Failed to download attachment ({}): {}", res.status(), full_url);
        }

        let bytes = res.bytes()?;
        std::fs::write(dest_path, bytes)?;
        Ok(())
    }
}
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for b in s.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(b as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
        result
    }
}

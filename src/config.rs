pub const DEFAULT_HOST: &str = "http://localhost:3000";
pub const PRESET_HOST_LOCAL: &str = "http://localhost:3000";
pub const PRESET_HOST_DOCKER: &str = "http://localhost:3011";

pub fn get_preset_hosts() -> Vec<String> {
    let mut hosts = Vec::new();
    if let Ok(remote) = std::env::var("MEMORABLE_REMOTE_URL").or_else(|_| std::env::var("MEMORABLE_URL")) {
        let norm = normalize_url(&remote);
        if !norm.is_empty() && !hosts.contains(&norm) {
            hosts.push(norm);
        }
    }

    for preset in [PRESET_HOST_LOCAL, PRESET_HOST_DOCKER] {
        let p = preset.to_string();
        if !hosts.contains(&p) {
            hosts.push(p);
        }
    }

    hosts
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub api_token: Option<String>,
    pub allow_host_switcher: bool,
}

pub fn normalize_url(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return DEFAULT_HOST.to_string();
    }
    if !s.starts_with("http://") && !s.starts_with("https://") {
        s = format!("http://{}", s);
    }
    while s.ends_with('/') {
        s.pop();
    }
    if s.ends_with("/api/v1") {
        s.truncate(s.len() - 7);
    }
    while s.ends_with('/') {
        s.pop();
    }
    s
}

impl Config {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: normalize_url(base_url),
            api_token: std::env::var("MEMORABLE_API_KEY")
                .or_else(|_| std::env::var("MEMORABLE_API_TOKEN"))
                .ok(),
            allow_host_switcher: std::env::var("MEMORABLE_REMOTE_URL")
                .or_else(|_| std::env::var("MEMORABLE_URL"))
                .is_err(),
        }
    }

    pub fn from_env_and_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let env_remote = std::env::var("MEMORABLE_REMOTE_URL")
            .or_else(|_| std::env::var("MEMORABLE_URL"))
            .ok();
        Self::parse_from_args_and_env(&args, env_remote.as_deref())
    }

    pub fn parse_from_args_and_env(args: &[String], env_remote: Option<&str>) -> Self {
        let mut chosen_url: Option<String> = None;
        let mut has_allow_switch = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--url" | "--host" | "-u" => {
                    if i + 1 < args.len() {
                        chosen_url = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                arg if arg.starts_with("--url=") => {
                    chosen_url = Some(arg.trim_start_matches("--url=").to_string());
                }
                arg if arg.starts_with("--host=") => {
                    chosen_url = Some(arg.trim_start_matches("--host=").to_string());
                }
                "--local" => {
                    chosen_url = Some(PRESET_HOST_LOCAL.to_string());
                }
                "--remote" => {
                    if let Some(remote) = env_remote {
                        chosen_url = Some(remote.to_string());
                    }
                }
                "--allow-switch" => {
                    has_allow_switch = true;
                }
                _ => {}
            }
            i += 1;
        }

        let base_url = chosen_url
            .or_else(|| env_remote.map(|s| s.to_string()))
            .unwrap_or_else(|| DEFAULT_HOST.to_string());

        let allow_host_switcher = env_remote.is_none() || has_allow_switch;

        Self {
            base_url: normalize_url(&base_url),
            api_token: std::env::var("MEMORABLE_API_KEY")
                .or_else(|_| std::env::var("MEMORABLE_API_TOKEN"))
                .ok(),
            allow_host_switcher,
        }
    }

    pub fn set_url(&mut self, new_url: &str) {
        self.base_url = normalize_url(new_url);
    }
}

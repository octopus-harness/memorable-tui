const MONTHS: [(&str, &str, &str); 12] = [
    ("Jan", "January", "01"),
    ("Feb", "February", "02"),
    ("Mar", "March", "03"),
    ("Apr", "April", "04"),
    ("May", "May", "05"),
    ("Jun", "June", "06"),
    ("Jul", "July", "07"),
    ("Aug", "August", "08"),
    ("Sep", "September", "09"),
    ("Oct", "October", "10"),
    ("Nov", "November", "11"),
    ("Dec", "December", "12"),
];

struct Timestamp<'a> {
    date: &'a str,
    year: &'a str,
    month: &'a str,
    day: &'a str,
    hhmm: Option<&'a str>,
}

impl<'a> Timestamp<'a> {
    fn parse(s: &'a str) -> Option<Self> {
        if s.len() < 10 {
            return None;
        }
        let date = s.get(..10)?;
        let mut parts = date.split('-');
        let year = parts.next()?;
        let month = parts.next()?;
        let day = parts.next()?;
        if parts.next().is_some() {
            return None;
        }
        let hhmm = if s.len() >= 16 { s.get(11..16) } else { None };
        Some(Self {
            date,
            year,
            month,
            day,
            hhmm,
        })
    }

    fn month_short(&self) -> &str {
        month_lookup(self.month)
            .map(|i| MONTHS[i].0)
            .unwrap_or(self.month)
    }
}

fn month_lookup(month: &str) -> Option<usize> {
    MONTHS.iter().position(|(.., num)| *num == month)
}

pub fn date_short(s: &str) -> String {
    match Timestamp::parse(s) {
        Some(ts) => format!(
            "{} {}, {}",
            ts.month_short(),
            ts.day.trim_start_matches('0'),
            ts.year
        ),
        None => s.to_string(),
    }
}

pub fn datetime_compact(s: &str) -> String {
    match Timestamp::parse(s) {
        Some(ts) if ts.hhmm.is_some() => format!("{} {}", ts.date, ts.hhmm.unwrap()),
        _ => s.to_string(),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn word_and_line_count(text: &str) -> (usize, usize) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (0, 0);
    }
    let words = trimmed.split_whitespace().count();
    let lines = text.lines().count();
    (words, lines)
}

pub fn similarity_percentage(score: Option<f64>) -> String {
    match score {
        Some(s) => format!("{:.1}% match", s * 100.0),
        None => "keyword match".to_string(),
    }
}

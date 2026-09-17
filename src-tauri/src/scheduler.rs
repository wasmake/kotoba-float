use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtitle {
    pub session_id: String,
    pub segment_id: u64,
    pub start_ms: u64,
    pub end_ms: u64,
    pub japanese: String,
    pub romaji: Option<String>,
    pub translation: Option<String>,
    pub transcript_latency_ms: Option<u64>,
    pub translation_latency_ms: Option<u64>,
}
pub fn deduplicate(previous: &str, next: &str) -> String {
    let max = 24.min(previous.chars().count()).min(next.chars().count());
    for n in (2..=max).rev() {
        let p: String = previous
            .chars()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let q: String = next.chars().take(n).collect();
        if p == q {
            return next.chars().skip(n).collect();
        }
    }
    next.into()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_boundary_overlap() {
        assert_eq!(deduplicate("今日はいい天気", "いい天気ですね"), "ですね")
    }
    #[test]
    fn leaves_distinct_text() {
        assert_eq!(deduplicate("はい", "そうです"), "そうです")
    }
}

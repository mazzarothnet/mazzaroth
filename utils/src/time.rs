#[allow(clippy::unwrap_used)]
pub fn get_current_time_ms() -> u64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

use client::{
    fmt_kv,
    LogColor,
};

pub fn format_timestamped_log(log_message: impl ToString) -> String {
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, false);
    let timestamp_str = format!("[{timestamp}]");

    fmt_kv!(timestamp_str, log_message, LogColor::Gray, LogColor::Debug)
}

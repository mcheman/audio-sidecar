use std::time::Duration;
use log::error;

pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs_f64();

    let minutes = (seconds / 60.0).floor();
    let seconds = seconds % 60.0;

    let hours = (minutes / 60.0).floor();
    let minutes = minutes % 60.0;

    if hours > 0.0 {
        format!("{}h {}m {:.1}s", hours, minutes, seconds)
    } else if minutes > 0.0 {
        format!("{}m {:.1}s", minutes, seconds)
    } else {
        format!("{:.1}s", seconds)
    }
}

pub fn or_die(result: Result<(), String>) {
    if let Err(msg) = result {
        die(format!("Something weird happened because a function that should not have failed has failed: {}", msg).as_str());
    }
}

pub fn die(s: &str) -> ! {
    error!("{}", s);
    std::panic!();
}
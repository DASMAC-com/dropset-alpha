use std::{
    collections::VecDeque,
    sync::LazyLock,
};

use client::{
    logs::fmt_divider,
    LogColor,
};
use colored::{
    Color,
    Colorize,
};

/// Number of recent log lines kept above the depth chart.
const LOG_LINES: usize = 8;

pub const BUY_COLOR: Color = Color::TrueColor {
    r: 30,
    g: 135,
    b: 80,
};
pub const SELL_COLOR: Color = Color::TrueColor {
    r: 240,
    g: 75,
    b: 90,
};

pub fn divider() -> String {
    static DIVIDER: LazyLock<String> =
        LazyLock::new(|| fmt_divider().color(LogColor::FadedGray).to_string());

    LazyLock::force(&DIVIDER).clone()
}

/// A terminal logger with two modes:
///
/// - `visualize = true`: fixed-height in-place display with a scrolling log buffer above a sticky
///   depth chart. The total rendered height is `2 + LOG_LINES + chart lines`.
/// - `visualize = false`: plain scrolling output via `println!`, no ANSI cursor movement.
pub struct Logger {
    visualize: bool,
    log_buffer: VecDeque<String>,
    chart: Vec<String>,
    /// Lines rendered in the last draw — used to move the cursor back to the top.
    rendered_lines: usize,
}

impl Logger {
    pub fn new(visualize: bool) -> Self {
        Self {
            visualize,
            log_buffer: VecDeque::new(),
            chart: Vec::new(),
            rendered_lines: 0,
        }
    }

    /// Append a log line. In visualize mode this redraws the fixed block; otherwise log the info
    /// with `tracing`.
    pub fn log(&mut self, line: String) {
        if !self.visualize {
            tracing::info!("{line}");
            return;
        }
        self.log_buffer.push_back(line);
        if self.log_buffer.len() > LOG_LINES {
            self.log_buffer.pop_front();
        }
        self.redraw();
    }

    /// Replace the depth chart and redraw. No-op when not in visualize mode, or if the log
    /// buffer is empty (render is deferred until the first log line arrives).
    pub fn update_chart(&mut self, lines: Vec<String>) {
        if !self.visualize {
            return;
        }
        self.chart = lines;
        self.redraw();
    }

    fn redraw(&mut self) {
        // Don't render anything until there's at least one log line — avoids showing
        // empty dividers and padding before the bot has produced any output.
        if self.log_buffer.is_empty() && self.chart.is_empty() {
            return;
        }
        if self.log_buffer.is_empty() {
            // Chart is present but no logs yet — skip the log section entirely so no
            // empty dividers appear. Nothing has been rendered yet so there's nothing to overwrite.
            return;
        }

        if self.rendered_lines > 0 {
            print!("\x1b[{}A", self.rendered_lines);
        }

        println!("\x1b[2K{}", divider());
        // Always render exactly LOG_LINES lines so the block height is fixed.
        // Empty lines pad the top until the buffer fills up.
        for _ in 0..LOG_LINES.saturating_sub(self.log_buffer.len()) {
            println!("\x1b[2K");
        }
        for line in &self.log_buffer {
            println!("\x1b[2K{line}");
        }
        println!("\x1b[2K{}", divider());
        for line in &self.chart {
            println!("\x1b[2K{line}");
        }
        // 2 dividers + LOG_LINES log rows (always) + chart rows
        self.rendered_lines = 2 + LOG_LINES + self.chart.len();
    }
}

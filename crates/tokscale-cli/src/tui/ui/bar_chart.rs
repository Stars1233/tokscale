use ratatui::prelude::*;

use super::widgets::format_tokens;
use crate::tui::app::App;

const BAR_CHARS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const MONTH_NAMES: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn render_bar_chart(frame: &mut Frame, app: &App, area: Rect, data: &[(String, f64, Color)]) {
    if data.is_empty() {
        return;
    }

    let is_very_narrow = app.is_very_narrow();
    let y_label_width: u16 = if is_very_narrow { 6 } else { 7 };

    let chart_width = area.width.saturating_sub(y_label_width) as usize;
    let chart_height = area.height.saturating_sub(3) as usize;

    if chart_width == 0 || chart_height == 0 {
        return;
    }

    let max_value = data
        .iter()
        .map(|(_, v, _)| *v)
        .fold(0.0_f64, |a, b| a.max(b))
        .max(1.0);

    let buf = frame.buffer_mut();
    let bar_count = data.len();

    let get_bar_width = |index: usize| -> usize {
        if bar_count == 0 {
            return 1;
        }
        let start = (index * chart_width) / bar_count;
        let end = ((index + 1) * chart_width) / bar_count;
        (end - start).max(1)
    };

    let title = if is_very_narrow {
        "Tokens"
    } else {
        "Tokens per Day"
    };
    let title_y = area.y;
    for (i, ch) in title.chars().enumerate() {
        let x = area.x + y_label_width + i as u16;
        if x < area.x + area.width {
            buf[(x, title_y)]
                .set_char(ch)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
    }

    for row_from_bottom in (0..chart_height).rev() {
        let row_index = chart_height - 1 - row_from_bottom;
        let y = area.y + 1 + row_index as u16;

        let y_label = if row_from_bottom == chart_height - 1 {
            format_tokens(max_value as u64)
        } else {
            String::new()
        };
        let padded_label = format!("{:>width$}│", y_label, width = (y_label_width - 1) as usize);
        for (i, ch) in padded_label.chars().enumerate() {
            let x = area.x + i as u16;
            if x < area.x + y_label_width {
                buf[(x, y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(app.theme.muted));
            }
        }

        let mut x_pos = area.x + y_label_width;
        for (bar_index, (_, value, color)) in data.iter().enumerate() {
            let bar_width = get_bar_width(bar_index);

            let row_threshold = ((row_from_bottom + 1) as f64 / chart_height as f64) * max_value;
            let prev_threshold = (row_from_bottom as f64 / chart_height as f64) * max_value;
            let threshold_diff = row_threshold - prev_threshold;

            let (ch, fg_color) = if *value <= prev_threshold {
                (' ', app.theme.muted)
            } else if *value >= row_threshold {
                ('█', *color)
            } else {
                let ratio = if threshold_diff > 0.0 {
                    (value - prev_threshold) / threshold_diff
                } else {
                    1.0
                };
                let block_index = (ratio * 8.0).floor() as usize;
                let block_index = block_index.clamp(1, 8);
                (BAR_CHARS[block_index], *color)
            };

            for _ in 0..bar_width {
                if x_pos < area.x + area.width {
                    buf[(x_pos, y)].set_char(ch).set_fg(fg_color);
                    x_pos += 1;
                }
            }
        }
    }

    let axis_y = area.y + 1 + chart_height as u16;
    if axis_y < area.y + area.height {
        let zero_label = format!("{:>width$}│", "0", width = (y_label_width - 1) as usize);
        for (i, ch) in zero_label.chars().enumerate() {
            let x = area.x + i as u16;
            if x < area.x + y_label_width {
                buf[(x, axis_y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(app.theme.muted));
            }
        }
        for x in (area.x + y_label_width)..(area.x + area.width) {
            buf[(x, axis_y)]
                .set_char('─')
                .set_style(Style::default().fg(app.theme.muted));
        }
    }

    let label_y = axis_y + 1;
    if label_y < area.y + area.height && !data.is_empty() {
        let num_labels = if is_very_narrow { 2 } else { 3 };
        let label_interval = (bar_count / num_labels).max(1);

        for i in (0..bar_count).step_by(label_interval) {
            let (date_str, _, _) = &data[i];

            let label = if let Some((month_str, day_str)) = date_str.split_once('/') {
                if let (Ok(month), Ok(day)) = (month_str.parse::<usize>(), day_str.parse::<u32>()) {
                    if (1..=12).contains(&month) {
                        if is_very_narrow {
                            format!("{}/{}", month, day)
                        } else {
                            format!("{} {}", MONTH_NAMES[month - 1], day)
                        }
                    } else {
                        date_str.clone()
                    }
                } else {
                    date_str.clone()
                }
            } else {
                date_str.clone()
            };

            let bar_start_x = (i * chart_width) / bar_count;
            let label_x = area.x + y_label_width + bar_start_x as u16;

            for (j, ch) in label.chars().enumerate() {
                let x = label_x + j as u16;
                if x < area.x + area.width {
                    buf[(x, label_y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(app.theme.muted));
                }
            }
        }
    }
}

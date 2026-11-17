use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_rate_kbs(kb_sec: f64) -> String {
    if kb_sec >= 1024.0 {
        format!("{:.1} MB/s", kb_sec / 1024.0)
    } else if kb_sec > 0.0 {
        format!("{:.1} KB/s", kb_sec)
    } else {
        "0".to_string()
    }
}

pub fn format_uptime(secs: u64) -> String {
    if secs == 0 {
        return "0s".to_string();
    }

    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

pub fn normalize_key_char(c: char) -> char {
    match c {
        'q' | 'й' | 'Q' | 'Й' => 'q',
        'b' | 'и' | 'B' | 'И' => 'b',
        's' | 'ы' | 'S' | 'Ы' => 's',
        'j' | 'о' => 'j',
        'J' | 'О' => 'J',
        'k' | 'л' => 'k',
        'K' | 'Л' => 'K',
        'h' | 'р' | 'H' | 'Р' => 'h',
        'l' | 'д' | 'L' | 'Д' => 'l',
        'w' | 'ц' | 'W' | 'Ц' => 'w',
        'o' | 'щ' | 'O' | 'Щ' => 'o',
        'x' | 'ч' | 'X' | 'Ч' => 'x',
        'd' | 'в' | 'D' | 'В' => 'd',
        'y' | 'н' | 'Y' | 'Н' => 'y',
        'n' | 'т' | 'N' | 'Т' => 'n',
        't' | 'е' => 't',
        'T' | 'Е' => 'T',
        'g' | 'п' => 'g',
        'G' | 'П' => 'G',
        '/' | '.' => '/',
        'c' | 'с' | 'C' | 'С' => 'c',
        'm' | 'ь' | 'M' | 'Ь' => 'm',
        'p' | 'з' | 'P' | 'З' => 'p',
        '?' | ',' => '?',
        other => other,
    }
}

pub fn dim_background(buf: &mut Buffer, area: Rect) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(10 * 1024 * 1024), "10.0 MB");
        assert_eq!(format_bytes(4 * 1024 * 1024 * 1024), "4.00 GB");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(3665), "1h 1m 5s");
        assert_eq!(format_uptime(90061), "1d 1h 1m 1s");
    }

    #[test]
    fn test_normalize_key_char() {
        assert_eq!(normalize_key_char('й'), 'q');
        assert_eq!(normalize_key_char('Й'), 'q');
        assert_eq!(normalize_key_char('о'), 'j');
        assert_eq!(normalize_key_char('л'), 'k');
        assert_eq!(normalize_key_char('в'), 'd');
        assert_eq!(normalize_key_char('.'), '/');
        assert_eq!(normalize_key_char('z'), 'z');
    }
}
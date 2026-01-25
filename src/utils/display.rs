//! Terminal display utilities for robust CLI output formatting.
//!
//! This module provides utilities for formatting CLI output that handles
//! different screen sizes, Unicode text, and terminal capabilities.

use std::io::{self, IsTerminal};
use std::sync::OnceLock;
use terminal_size::terminal_size;

/// Terminal information with cached size and capabilities.
#[derive(Debug, Clone)]
pub struct Terminal {
    width: usize,
    height: usize,
    is_tty: bool,
}

static TERMINAL_INFO: OnceLock<Terminal> = OnceLock::new();

/// Get the global terminal information, initialized on first call.
pub fn terminal_info() -> &'static Terminal {
    TERMINAL_INFO.get_or_init(|| {
        let (width, height) = terminal_size()
            .map(|(w, h)| (w.0 as usize, h.0 as usize))
            .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));

        Terminal {
            width,
            height,
            is_tty: io::stdout().is_terminal(),
        }
    })
}

/// Default width when terminal size cannot be determined.
pub const DEFAULT_WIDTH: usize = 100;

/// Default height when terminal size cannot be determined.
pub const DEFAULT_HEIGHT: usize = 24;

/// Get the current terminal width in characters.
#[inline]
pub fn terminal_width() -> usize {
    terminal_info().width
}

/// Get the current terminal height in rows.
#[inline]
pub fn terminal_height() -> usize {
    terminal_info().height
}

/// Check if stdout is a terminal.
#[inline]
pub fn is_terminal() -> bool {
    terminal_info().is_tty
}

/// Clamp a value to the given range.
#[inline]
fn clamp(value: usize, min: usize, max: usize) -> usize {
    value.clamp(min, max)
}

/// Truncate text to fit within the specified width using unicode-aware truncation.
///
/// Returns a string that fits within `max_width` characters, cutting at word
/// boundaries when possible and appending an ellipsis if truncation occurred.
///
/// # Examples
///
/// ```
/// use research_master::utils::truncate_with_ellipsis;
///
/// // Simple truncation
/// assert_eq!(truncate_with_ellipsis("Hello World", 8), "Hello...");
/// assert_eq!(truncate_with_ellipsis("Hi", 8), "Hi");
/// ```
pub fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    // Use unicode-width to properly handle wide characters
    let char_widths: Vec<(char, usize)> = text
        .chars()
        .map(|c| (c, unicode_width::UnicodeWidthChar::width(c).unwrap_or(1)))
        .collect();

    let total_width: usize = char_widths.iter().map(|(_, w)| *w).sum();

    if total_width <= max_width {
        return text.to_string();
    }

    // Need to truncate - find the longest prefix that fits
    let mut current_width = 0;
    let mut end_idx = 0;

    for (i, (_c, w)) in char_widths.iter().enumerate() {
        if current_width + w > max_width.saturating_sub(3) {
            // We need 3 chars for ellipsis, stop before we exceed
            break;
        }
        current_width += w;
        end_idx = i + 1;
    }

    // If we couldn't fit even one character, just return empty with ellipsis
    if end_idx == 0 {
        return "...".to_string();
    }

    // Take only the characters that fit and add ellipsis
    let truncated: String = char_widths[..end_idx].iter().map(|(c, _)| *c).collect();
    format!("{}...", truncated)
}

/// Truncate text at word boundaries to fit within the specified width.
///
/// Unlike [`truncate_with_ellipsis`], this prefers to cut at the last complete
/// word before the width limit, which can result in shorter truncation but
/// more readable output.
///
/// # Examples
///
/// ```
/// use research_master::utils::truncate_at_word;
///
/// // Should cut at word boundary
/// let result = truncate_at_word("The quick brown fox", 12);
/// assert!(result.len() <= 15); // "The quick..." or "The..."
/// ```
pub fn truncate_at_word(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if text.len() <= max_width {
        return text.to_string();
    }

    // Try to find a word boundary near the limit
    let width_for_ellipsis = max_width.saturating_sub(3);

    // First try: find the last space before the width limit
    if let Some(last_space) = text[..width_for_ellipsis.min(text.len())].rfind(' ') {
        let candidate = &text[..last_space];
        // Check if this candidate fits
        let candidate_width: usize = candidate
            .chars()
            .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(1))
            .sum();
        if candidate_width <= width_for_ellipsis {
            return format!("{}...", candidate.trim_end());
        }
    }

    // Fall back to character-based truncation
    truncate_with_ellipsis(text, max_width)
}

/// Calculate column widths based on terminal width and column constraints.
///
/// Returns a vector of column widths that sum to at most `terminal_width`,
/// respecting the minimum and maximum constraints for each column.
///
/// # Arguments
///
/// * `terminal_width` - The total available width
/// * `columns` - A slice of `(min_width, preferred_width, max_width, weight)` tuples
///   - `min_width`: Minimum width the column must have
///   - `preferred_width`: Desired width if space allows
///   - `max_width`: Maximum width the column can expand to (usize::MAX for unlimited)
///   - `weight`: Relative weight for distributing extra space
///
/// # Examples
///
/// ```
/// use research_master::utils::calculate_column_widths;
///
/// // Title (min 30, preferred 50%, max 80, weight 2), Authors (min 20, preferred 30%, max 50, weight 1)
/// let widths = calculate_column_widths(100, &[(30, 50, 80, 2), (20, 30, 50, 1)]);
/// assert_eq!(widths.len(), 2);
/// ```
pub fn calculate_column_widths<const N: usize>(
    terminal_width: usize,
    columns: &[(usize, usize, usize, usize); N],
) -> [usize; N] {
    // Account for separators (N - 1 spaces between columns)
    let separator_width = N.saturating_sub(1);
    let available_width = terminal_width.saturating_sub(separator_width);

    if available_width == 0 {
        return columns.map(|c| c.0); // All minimums
    }

    // First pass: ensure minimums
    let min_sum: usize = columns.iter().map(|c| c.0).sum();
    if min_sum >= available_width {
        // Can't even fit minimums, return minimums
        return columns.map(|c| c.0);
    }

    // Calculate preferred sum and weights
    let mut widths = [0usize; N];
    let mut preferred_sum = 0usize;
    let mut total_weight = 0usize;

    for (i, (min, preferred, _, weight)) in columns.iter().enumerate() {
        let preferred = *preferred;
        let weight = *weight;
        preferred_sum += preferred;
        total_weight += weight;
        widths[i] = *min; // Start with minimums
    }

    // Second pass: distribute space based on preferences and weights
    if preferred_sum > available_width {
        // Need to compress towards minimums
        let compression_factor = available_width as f64 / preferred_sum as f64;
        for (i, (min, preferred, _, _)) in columns.iter().enumerate() {
            let target = (*preferred as f64 * compression_factor) as usize;
            widths[i] = clamp(target, *min, *min); // Can't expand, just use min
        }
    } else {
        // Can expand, distribute extra space
        let extra_space = available_width - min_sum;
        let mut remaining = extra_space;

        // First round: give each column up to its preferred width
        for (i, (min, preferred, _, _)) in columns.iter().enumerate() {
            let can_give = preferred.saturating_sub(*min);
            let to_give = std::cmp::min(can_give, remaining);
            widths[i] = min + to_give;
            remaining -= to_give;
        }

        // Second round: distribute remaining space by weight
        if remaining > 0 && total_weight > 0 {
            for (i, (_, _, _, weight)) in columns.iter().enumerate() {
                if remaining == 0 {
                    break;
                }
                let share = (remaining * *weight) / total_weight;
                if share > 0 {
                    widths[i] += share;
                    remaining -= share;
                }
            }
        }

        // Third round: distribute any remaining (due to rounding) by max constraint
        for (i, (_, _, max, _)) in columns.iter().enumerate() {
            if remaining == 0 {
                break;
            }
            if *max != usize::MAX {
                let can_take = max.saturating_sub(widths[i]);
                let to_take = std::cmp::min(can_take, remaining);
                widths[i] += to_take;
                remaining -= to_take;
            }
        }
    }

    widths
}

/// Column width configuration for table display.
#[derive(Debug, Clone, Copy)]
pub struct ColumnConfig {
    pub min_width: usize,
    pub max_width: usize,
    pub weight: usize,
}

impl Default for ColumnConfig {
    fn default() -> Self {
        ColumnConfig {
            min_width: 1,
            max_width: usize::MAX,
            weight: 1,
        }
    }
}

impl ColumnConfig {
    /// Create a new column config with minimum width.
    pub fn new(min_width: usize) -> Self {
        ColumnConfig {
            min_width,
            max_width: usize::MAX,
            weight: 1,
        }
    }

    /// Set the maximum width.
    pub fn max(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    /// Set the weight for space distribution.
    pub fn weight(mut self, weight: usize) -> Self {
        self.weight = weight;
        self
    }
}

/// Calculate column widths from a list of column configurations.
pub fn calculate_dynamic_column_widths(
    terminal_width: usize,
    configs: &[ColumnConfig],
) -> Vec<usize> {
    let n = configs.len();
    if n == 0 {
        return vec![];
    }

    // Account for separators
    let separator_width = n.saturating_sub(1);
    let available_width = terminal_width.saturating_sub(separator_width);

    if available_width == 0 {
        return configs.iter().map(|c| c.min_width).collect();
    }

    // Calculate minimum sum
    let min_sum: usize = configs.iter().map(|c| c.min_width).sum();
    if min_sum >= available_width {
        return configs.iter().map(|c| c.min_width).collect();
    }

    // Calculate preferred widths as the midpoint between min and max (or min if max is unlimited)
    let preferred_widths: Vec<usize> = configs
        .iter()
        .map(|c| {
            if c.max_width == usize::MAX {
                c.min_width * 2 + 10 // Some reasonable preferred width
            } else {
                (c.min_width + c.max_width) / 2
            }
        })
        .collect();

    let preferred_sum: usize = preferred_widths.iter().sum();
    let total_weight: usize = configs.iter().map(|c| c.weight).sum();

    let mut widths: Vec<usize> = configs.iter().map(|c| c.min_width).collect();
    let mut remaining = available_width - min_sum;

    if preferred_sum > available_width {
        // Need to compress
        let compression_factor = available_width as f64 / preferred_sum as f64;
        for (i, config) in configs.iter().enumerate() {
            let target = (preferred_widths[i] as f64 * compression_factor) as usize;
            widths[i] = clamp(target, config.min_width, config.max_width);
        }
    } else {
        // First pass: expand to preferred
        for (i, config) in configs.iter().enumerate() {
            let target = preferred_widths[i];
            let expansion = (target - config.min_width).min(remaining);
            widths[i] = config.min_width + expansion;
            remaining -= expansion;
        }

        // Second pass: distribute remaining by weight
        if remaining > 0 && total_weight > 0 {
            for (i, config) in configs.iter().enumerate() {
                if remaining == 0 {
                    break;
                }
                let share = (remaining * config.weight) / total_weight;
                if share > 0 {
                    let can_expand = if config.max_width == usize::MAX {
                        remaining
                    } else {
                        config.max_width.saturating_sub(widths[i])
                    };
                    let to_take = share.min(can_expand).min(remaining);
                    widths[i] += to_take;
                    remaining -= to_take;
                }
            }
        }
    }

    widths
}

/// Format a paper title for display, truncating if necessary.
pub fn format_title(title: &str, max_width: usize) -> String {
    if max_width <= 3 {
        return "...".to_string();
    }
    truncate_at_word(title, max_width)
}

/// Format authors for display, truncating if necessary.
pub fn format_authors(authors: &str, max_width: usize) -> String {
    if max_width <= 3 {
        return "...".to_string();
    }
    truncate_with_ellipsis(authors, max_width)
}

/// Format a source name for display.
pub fn format_source(source: &str, max_width: usize) -> String {
    truncate_with_ellipsis(source, max_width)
}

/// Format a year for display.
pub fn format_year(year: &str) -> String {
    year.chars().take(4).collect()
}

/// Get optimal column widths for a paper table.
///
/// Returns (title_width, authors_width, source_width, year_width).
pub fn get_paper_table_columns(terminal_width: usize) -> (usize, usize, usize, usize) {
    // Column configuration for papers table
    // Title: most important, takes 50% of space
    // Authors: important, takes 25% of space
    // Source: fixed width around 12 chars
    // Year: fixed width around 4 chars
    let configs = [
        ColumnConfig::new(30).max(80).weight(2),
        ColumnConfig::new(20).max(50).weight(1),
        ColumnConfig::new(8).max(15).weight(0),
        ColumnConfig::new(4).max(6).weight(0),
    ];

    let widths = calculate_dynamic_column_widths(terminal_width, &configs);

    if widths.len() == 4 {
        (widths[0], widths[1], widths[2], widths[3])
    } else {
        // Fallback defaults
        (
            (terminal_width as f64 * 0.50) as usize,
            (terminal_width as f64 * 0.25) as usize,
            12,
            4,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_with_ellipsis_basic() {
        assert_eq!(truncate_with_ellipsis("Hello", 10), "Hello");
        assert_eq!(truncate_with_ellipsis("Hello World", 8), "Hello...");
    }

    #[test]
    fn test_truncate_with_ellipsis_empty() {
        assert_eq!(truncate_with_ellipsis("", 10), "");
        assert_eq!(truncate_with_ellipsis("Hello", 0), "");
        assert_eq!(truncate_with_ellipsis("Hello", 1), "...");
    }

    #[test]
    fn test_truncate_at_word() {
        let result = truncate_at_word("The quick brown fox", 10);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 13); // "The quick..."
    }

    #[test]
    fn test_calculate_column_widths_basic() {
        // Two columns, 50 width terminal
        let cols = [
            (10usize, 30usize, 40usize, 1),
            (10usize, 30usize, 40usize, 1),
        ];
        let widths = calculate_column_widths::<2>(50, &cols);
        assert_eq!(widths.len(), 2);
        // Should sum to <= 50 (minus separator)
        assert!(widths.iter().sum::<usize>() <= 50);
    }

    #[test]
    fn test_calculate_column_widths_min_exceeded() {
        // Two columns with minimums summing to more than terminal width
        let cols = [
            (30usize, 30usize, usize::MAX, 1),
            (30usize, 30usize, usize::MAX, 1),
        ];
        let widths = calculate_column_widths::<2>(50, &cols);
        assert_eq!(widths, [30, 30]); // Should be minimums
    }

    #[test]
    fn test_get_paper_table_columns() {
        let (title, authors, source, year) = get_paper_table_columns(100);
        assert!(title > 0);
        assert!(authors > 0);
        assert!(source > 0);
        assert!(year > 0);
        // Should fit in terminal
        assert!(title + authors + source + year + 3 <= 100);
    }

    #[test]
    fn test_format_title() {
        assert_eq!(format_title("Hello World", 10), "Hello...");
        assert_eq!(format_title("Hi", 10), "Hi");
    }

    #[test]
    fn test_format_year() {
        assert_eq!(format_year("2023-05-15"), "2023");
        assert_eq!(format_year("2023"), "2023");
        assert_eq!(format_year(""), "");
    }
}

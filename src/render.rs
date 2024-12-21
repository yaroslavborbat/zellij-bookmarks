use crate::{BASE_COLOR, RESERVE_ROW_COUNT};
use owo_colors::OwoColorize;
use zellij_tile::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_main_menu<'a>(
    rows: usize,
    cols: usize,
    selected: usize,
    count: usize,
    mode: String,
    filter: String,
    filter_by: String,
    iterator: impl Iterator<Item = (usize, usize, &'a String)>,
) {
    let (x, y, width, height) = main_menu_size(rows, cols);

    render_mode(x + 2, y, mode);

    render_search_block(x + 2, y + 1, filter, filter_by);

    let (begin, end) = if selected >= height {
        (selected + 1 - height, selected)
    } else {
        (0, height - 1)
    };

    render_right_counter(begin, width, y + 2);

    {
        let mut number = y + 3;

        for (i, id, value) in iterator {
            if i < begin {
                continue;
            }
            if i > end {
                break;
            }
            let text = prepare_row_text(value.clone(), id, width, selected == i);

            print_text_with_coordinates(text, x, number, None, None);

            number += 1;
        }
    }

    render_all_counter(x + 2, rows, count);

    if count > end {
        render_right_counter_with_max(count - 1 - end, count, width, rows);
    }
}

fn main_menu_size(rows: usize, cols: usize) -> (usize, usize, usize, usize) {
    // x, y, width, height
    let width = cols;
    let x = 0;
    let y = 0;
    let height = rows.saturating_sub(RESERVE_ROW_COUNT);

    (x, y, width, height)
}

fn prepare_row_text(row: String, id: usize, max_length: usize, selected: bool) -> Text {
    let truncated_row = {
        let formatted = format!("{}. {}", id, row);
        if formatted.len() > max_length {
            let truncated_len = max_length.saturating_sub(3);
            let mut truncated_str = formatted.chars().take(truncated_len).collect::<String>();
            truncated_str.push_str("...");
            truncated_str
        } else {
            formatted
        }
    };
    let text = if selected {
        Text::new(truncated_row.yellow().to_string()).selected()
    } else {
        Text::new(truncated_row)
    };
    text
}

pub(crate) fn render_mode(x: usize, y: usize, mode: String) {
    let s = format!("Mode: {}", mode);
    let text = Text::new(s).color_range(BASE_COLOR, ..4);
    print_text_with_coordinates(text, x, y, None, None)
}

fn render_search_block(x: usize, y: usize, filter: String, filter_by: String) {
    let filter = format!("Search (by {}) {}_", filter_by, filter.clone());

    let text = Text::new(filter).color_range(BASE_COLOR, ..6);
    print_text_with_coordinates(text, x, y, None, None);
}

// Render row with All row-counter
fn render_all_counter(x: usize, y: usize, all: usize) {
    let all_count = format!("All: {}", all);
    let text = Text::new(all_count).color_range(BASE_COLOR, ..);
    print_text_with_coordinates(text, x, y, None, None);
}

// Render row with right counter with max
fn render_right_counter_with_max(count: usize, max_count: usize, width: usize, y: usize) {
    if count == max_count {
        return;
    }
    render_right_counter(count, width, y);
}

// Render row with right counter
fn render_right_counter(count: usize, width: usize, y: usize) {
    if count == 0 {
        return;
    }
    let row = format!("+ {} more  ", count);
    let x = width - row.len();
    let text = Text::new(row.yellow().bold().to_string());
    print_text_with_coordinates(text, x, y, None, None);
}

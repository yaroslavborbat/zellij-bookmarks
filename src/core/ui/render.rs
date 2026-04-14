use std::collections::HashSet;
use zellij_tile::prelude::*;

pub const RESERVE_ROW_COUNT: usize = 6;
pub const RESERVE_COLUMN_COUNT: usize = 36;

#[derive(Clone, Copy, Debug)]
pub struct UiStyle {
    pub chrome_color: usize,
    pub match_color: usize,
    pub active_item_color: usize,
    pub selected_item_frame: bool,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            chrome_color: 2,
            match_color: 3,
            active_item_color: 0,
            selected_item_frame: true,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_main_menu<'a, T: std::fmt::Display + PartialEq + Copy>(
    rows: usize,
    cols: usize,
    selected: usize,
    count: usize,
    mode: T,
    all_modes: &[T],
    ui_style: &UiStyle,
    filter: String,
    filter_by: String,
    iterator: impl Iterator<Item = (usize, usize, &'a String, Vec<usize>)>,
) {
    let (x, y, width, height) = main_menu_size(rows, cols);

    render_mode(x, y, mode, all_modes, ui_style);

    render_search_block(x + 2, y + 2, filter, filter_by, ui_style);

    let (begin, end) = if selected >= height {
        (selected + 1 - height, selected)
    } else {
        (0, height.saturating_sub(1))
    };

    render_right_counter(begin, width, y + 3, ui_style);

    {
        let mut number = y + 4;

        for (i, id, value, indices) in iterator {
            if i < begin {
                continue;
            }
            if i > end {
                break;
            }
            let text = prepare_row_text(value.clone(), id, width, selected == i, indices, ui_style);

            print_text_with_coordinates(text, x, number, None, None);

            number += 1;
        }
    }

    render_all_counter(x + 2, rows, count, ui_style);

    if count > end {
        render_right_counter_with_max(count - 1 - end, count, width, rows, ui_style);
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

fn prepare_row_text(
    row: String,
    id: usize,
    max_length: usize,
    selected: bool,
    indices: Vec<usize>,
    ui_style: &UiStyle,
) -> Text {
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

    let mut row_text = Text::new(truncated_row);
    if selected && ui_style.selected_item_frame {
        row_text = row_text.selected()
    }
    let fix_id_shift = id.to_string().len() + 2;

    let new_indices: HashSet<usize> = indices.iter().map(|i| i + fix_id_shift).collect();

    for i in 0..row_text.len() {
        if new_indices.contains(&i) {
            row_text = row_text.color_range(ui_style.match_color, i..i + 1);
        } else if selected {
            row_text = row_text.color_range(ui_style.active_item_color, i..i + 1);
        }
    }

    row_text
}

pub fn render_mode<T: std::fmt::Display + PartialEq + Copy>(
    x: usize,
    y: usize,
    mode: T,
    all_modes: &[T],
    ui_style: &UiStyle,
) {
    let key_indication_text = format!("{}{}", BareKey::Left, BareKey::Right);
    let mut shift = x + key_indication_text.chars().count() + 1;

    print_text_with_coordinates(
        Text::new(key_indication_text)
            .color_range(ui_style.chrome_color, ..)
            .opaque(),
        x,
        y,
        None,
        None,
    );

    all_modes.iter().for_each(|m| {
        let mut t = Text::new(m.to_string());
        if *m == mode {
            t = t.selected();
        };

        print_ribbon_with_coordinates(t, x + shift, y, None, None);
        shift += m.to_string().len() + 4;
    });
}

fn render_search_block(x: usize, y: usize, filter: String, filter_by: String, ui_style: &UiStyle) {
    let filter = format!("Search (by {}): {}_", filter_by, filter.clone());

    let text = Text::new(filter).color_range(ui_style.chrome_color, ..6);
    print_text_with_coordinates(text, x, y, None, None);
}

// Render row with All row-counter
fn render_all_counter(x: usize, y: usize, all: usize, ui_style: &UiStyle) {
    let all_count = format!("All: {}", all);
    let text = Text::new(all_count).color_range(ui_style.chrome_color, ..);
    print_text_with_coordinates(text, x, y, None, None);
}

// Render row with right counter with max
fn render_right_counter_with_max(
    count: usize,
    max_count: usize,
    width: usize,
    y: usize,
    ui_style: &UiStyle,
) {
    if count == max_count {
        return;
    }
    render_right_counter(count, width, y, ui_style);
}

// Render row with right counter
fn render_right_counter(count: usize, width: usize, y: usize, ui_style: &UiStyle) {
    if count == 0 {
        return;
    }
    let row = format!("+ {} more  ", count);
    let x = width - row.len();
    let text = Text::new(row).color_range(ui_style.chrome_color, ..);
    print_text_with_coordinates(text, x, y, None, None);
}

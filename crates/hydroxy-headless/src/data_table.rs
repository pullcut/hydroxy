//! Data table connector. Anatomy: root, scroller, table, caption, header,
//! header-cell, sort-button, body, row, cell, empty, footer. Every part
//! carries `data-scope="data-table"` and its `data-part`. Sorted header cells
//! carry `aria-sort`; selected rows `aria-selected` and `data-selected`.

pub use hydroxy_machines::data_table::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "data-table";

/// The direction `key` is sorted in, if it is the sorted column.
#[must_use]
pub fn sort_direction(state: &DataTableState, key: &str) -> Option<SortDirection> {
    state.sort.as_ref().filter(|s| s.key == key).map(|s| s.direction)
}

/// `aria-sort` for a sortable header cell.
#[must_use]
pub fn aria_sort(state: &DataTableState, key: &str) -> &'static str {
    match sort_direction(state, key) {
        Some(SortDirection::Ascending) => "ascending",
        Some(SortDirection::Descending) => "descending",
        None => "none",
    }
}

/// `data-sort` on the sort button: `asc`, `desc` or `none`.
#[must_use]
pub fn data_sort(state: &DataTableState, key: &str) -> &'static str {
    match sort_direction(state, key) {
        Some(SortDirection::Ascending) => "asc",
        Some(SortDirection::Descending) => "desc",
        None => "none",
    }
}

/// Whether row `id` is selected.
#[must_use]
pub fn is_selected(state: &DataTableState, id: &str) -> bool {
    state.selected.iter().any(|s| s == id)
}

/// The footer's range text, such as `11–20 of 42`.
#[must_use]
pub fn range_label(page: usize, page_size: Option<usize>, len: usize) -> String {
    let range = page_range(page, page_size, len);
    if range.is_empty() { format!("0 of {len}") } else { format!("{}–{} of {len}", range.start + 1, range.end) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aria_sort_only_on_the_sorted_column() {
        let state = DataTableState { sort: Some(Sort::desc("age")), ..Default::default() };
        assert_eq!(aria_sort(&state, "age"), "descending");
        assert_eq!(aria_sort(&state, "name"), "none");
        assert_eq!(data_sort(&state, "age"), "desc");
    }

    #[test]
    fn range_label_reads_naturally() {
        assert_eq!(range_label(1, Some(10), 42), "11–20 of 42");
        assert_eq!(range_label(0, Some(10), 0), "0 of 0");
    }
}

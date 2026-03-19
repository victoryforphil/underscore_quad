pub fn filter_items(items: &[String], query: &str) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return (0..items.len()).collect();
    }

    items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            if item.to_lowercase().contains(&q) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

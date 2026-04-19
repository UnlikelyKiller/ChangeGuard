use comfy_table::Table;

pub fn build_table(headers: impl IntoIterator<Item = impl ToString>) -> Table {
    let mut table = Table::new();
    table.set_header(
        headers
            .into_iter()
            .map(|header| header.to_string())
            .collect::<Vec<_>>(),
    );
    table
}

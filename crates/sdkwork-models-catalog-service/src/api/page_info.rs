use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApiPageInfo {
    pub mode: &'static str,
    pub page: i64,
    pub page_size: i64,
    pub total_items: String,
    pub total_pages: i64,
    pub has_more: bool,
}

pub fn offset_page_info(page: i64, page_size: i64, total_items: i64) -> ApiPageInfo {
    let page = page.max(1);
    let page_size = page_size.max(1);
    let total_items = total_items.max(0);
    let total_pages = if total_items == 0 {
        0
    } else {
        (total_items + page_size - 1) / page_size
    };
    ApiPageInfo {
        mode: "offset",
        page,
        page_size,
        total_items: total_items.to_string(),
        total_pages,
        has_more: page < total_pages,
    }
}

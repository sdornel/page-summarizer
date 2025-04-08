use scraper::{Html, Selector};
use scraper::ElementRef;

/// Extracts main content including paragraphs and code samples from an HTML document,
/// while excluding content from certain tags (nav, header, footer, etc.).
pub fn extract_main_content(html: &str) -> String {
    let document = Html::parse_document(html);

    // Selectors for content we want to capture.
    // This now includes paragraphs, preformatted text, and inline code.
    let content_selectors = vec![
        Selector::parse("p").unwrap(),
        Selector::parse("pre").unwrap(),
        Selector::parse("code").unwrap(),
    ];

    // Selectors for elements to exclude.
    let exclude_selectors = vec![
        Selector::parse("nav").unwrap(),
        Selector::parse("footer").unwrap(),
        Selector::parse("header").unwrap(),
        Selector::parse("aside").unwrap(),
        Selector::parse("script").unwrap(),
        Selector::parse("style").unwrap(),
        Selector::parse("iframe").unwrap(),
        Selector::parse("noscript").unwrap(),
        Selector::parse("form").unwrap(),
        Selector::parse("input").unwrap(),
        Selector::parse("button").unwrap(),
        Selector::parse("select").unwrap(),
        Selector::parse("option").unwrap(),
        Selector::parse("label").unwrap(),
    ];

    let mut content_blocks = Vec::new();

    // For each content selector, collect elements that are not inside any excluded tags.
    for selector in content_selectors {
        for element in document.select(&selector) {
            let mut skip = false;
            // Check ancestors of the element.
            for ancestor in element.ancestors() {
                if let Some(ancestor_ref) = ElementRef::wrap(ancestor) {
                    for ex_sel in &exclude_selectors {
                        if ex_sel.matches(&ancestor_ref) {
                            skip = true;
                            break;
                        }
                    }
                }
                if skip {
                    break;
                }
            }
            if !skip {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if text.trim().len() > 20 {
                    content_blocks.push(text);
                }
            }
        }
    }

    // Join all the content blocks.
    let combined = content_blocks.join(" ");

    // Clean up the text: remove control characters and normalize whitespace.
    let cleaned = combined
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    cleaned
}
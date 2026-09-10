// Portable Text → HTML renderer.
//
// Sanity stores rich text as a JSON array of typed blocks (Portable Text).
// This module traverses that structure and produces HTML.

use crate::sanity::models::{PtBlock, PtChild, PtMarkDef};

/// Render a list of Portable Text blocks into an HTML string.
pub fn render(blocks: &[PtBlock], project_id: &str, dataset: &str) -> String {
    let mut html = String::new();
    for block in blocks {
        html.push_str(&render_block(block, project_id, dataset));
    }
    html
}

/// Dispatch a single block to the appropriate renderer by its `_type`.
fn render_block(block: &PtBlock, project_id: &str, dataset: &str) -> String {
    match block._type.as_str() {
        "block" => render_text_block(block),
        "image" => render_image(block, project_id, dataset),
        _ => String::new(),
    }
}

/// Render a text block (paragraph, heading, blockquote, etc.).
fn render_text_block(block: &PtBlock) -> String {
    let tag = match block.style.as_deref() {
        Some("h1") => "h1",
        Some("h2") => "h2",
        Some("h3") => "h3",
        Some("h4") => "h4",
        Some("h5") => "h5",
        Some("h6") => "h6",
        Some("blockquote") => "blockquote",
        _ => "p",
    };

    let children = block.children.as_deref().unwrap_or_default();
    let mark_defs = block.mark_defs.as_deref().unwrap_or_default();

    let inner: String = children
        .iter()
        .map(|child| render_child(child, mark_defs))
        .collect();

    format!("<{}>{}</{}>", tag, inner, tag)
}

/// Render an image block, resolving Sanity asset references to CDN URLs.
fn render_image(block: &PtBlock, project_id: &str, dataset: &str) -> String {
    let alt = block.alt.as_deref().unwrap_or("");
    let src = block
        .asset
        .as_ref()
        .and_then(|a| a._ref.as_ref())
        .map(|r#ref| {
            // ref format: "image-{hash}-{dims}-{ext}" — replace last dash with dot for extension
            let path = &r#ref[6..]; // skip "image-" prefix
            let url_path = match path.rfind('-') {
                Some(pos) => format!("{}.{}", &path[..pos], &path[pos + 1..]),
                None => path.to_string(),
            };
            format!(
                "https://cdn.sanity.io/images/{}/{}/{}",
                project_id, dataset, url_path
            )
        })
        .unwrap_or_default();
    format!(
        r#"<img src="{}" alt="{}" loading="lazy" class="rounded-lg" />"#,
        src, alt
    )
}

/// Render an inline text node, applying formatting marks (bold, italic, links, etc.).
///
/// Marks are identified by string keys:
///   "strong"       → `<strong>`
///   "em"           → `<em>`
///   "code"         → `<code>`
///   "link-{key}"   → `<a>` (resolved via `mark_defs`)
fn render_child(child: &PtChild, mark_defs: &[PtMarkDef]) -> String {
    let text = child.text.as_deref().unwrap_or("");
    if text.is_empty() {
        return String::new();
    }

    let mut html = escape_html(text);

    if let Some(marks) = &child.marks {
        for mark in marks {
            match mark.as_str() {
                "strong" => html = format!("<strong>{}</strong>", html),
                "em" => html = format!("<em>{}</em>", html),
                "underline" => html = format!("<u>{}</u>", html),
                "strike-through" => html = format!("<s>{}</s>", html),
                "code" => html = format!("<code>{}</code>", html),
                mark if mark.starts_with("link-") => {
                    let key = &mark[5..];
                    if let Some(mark_def) = mark_defs.iter().find(|md| md._key.as_deref() == Some(key)) {
                        if let Some(href) = &mark_def.href {
                            html = format!(r#"<a href="{}">{}</a>"#, escape_html(href), html);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    html
}

/// Escape HTML special characters to prevent XSS injection from user-authored content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }
}

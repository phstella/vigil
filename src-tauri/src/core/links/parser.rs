//! Regex-based link parser for markdown content.
//!
//! Extracts `[[wikilinks]]` and standard `[text](target.md)` links from
//! markdown content, skipping links inside fenced code blocks. Returns
//! structured [`ParsedLink`] records with source, target, link type, and
//! surrounding context.

use std::path::Path;

use regex::Regex;

use crate::models::links::LinkType;

/// A link extracted from markdown content before graph resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLink {
    /// Workspace-relative path of the source note.
    pub source_path: String,
    /// Raw link target (may be a stem like `other-note` or a relative path).
    pub raw_target: String,
    /// Resolved workspace-relative target path (with `.md` appended if needed).
    pub target_path: String,
    /// Whether this is a `[[wikilink]]` or `[text](path)` link.
    pub link_type: LinkType,
    /// The line containing the link, truncated to 200 characters.
    pub context: String,
    /// Optional fragment identifier (e.g., `#heading`).
    pub fragment: Option<String>,
}

/// Parse all links from markdown content.
///
/// Skips links inside fenced code blocks (triple-backtick). Handles:
/// - `[[target]]` and `[[target|display text]]` wikilinks
/// - `[[target#heading]]` wikilinks with fragments
/// - `[text](path.md)` and `[text](./path.md)` markdown links
/// - `[text](path.md#heading)` markdown links with fragments
pub fn parse_links(content: &str, source_path: &str) -> Vec<ParsedLink> {
    let stripped = strip_code_blocks(content);
    let mut links = Vec::new();

    parse_wikilinks(&stripped, source_path, &mut links);
    parse_markdown_links(&stripped, source_path, &mut links);

    links
}

/// Strip fenced code blocks from content, replacing them with empty lines
/// to preserve line numbers for context extraction.
fn strip_code_blocks(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            result.push('\n');
        } else if in_code_block {
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Parse `[[wikilinks]]` from content.
fn parse_wikilinks(content: &str, source_path: &str, links: &mut Vec<ParsedLink>) {
    let re = Regex::new(r"\[\[([^\]\n]+?)]]").expect("invalid wikilink regex");
    let source_dir = Path::new(source_path).parent();

    for cap in re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let link_content = &cap[1];

        // Split on `|` for display text: [[target|display]]
        let target_part = link_content.split('|').next().unwrap_or("").trim();
        if target_part.is_empty() {
            continue;
        }

        // Split on `#` for fragment: [[target#heading]]
        let (stem, fragment) = split_fragment(target_part);
        if stem.is_empty() && fragment.is_some() {
            // Pure fragment link like [[#heading]] - skip, it's an internal link
            continue;
        }

        // Resolve target path
        let target_path = resolve_wikilink_target(&stem, source_dir);

        let context = extract_context(content, full_match.start());

        links.push(ParsedLink {
            source_path: source_path.to_string(),
            raw_target: target_part.to_string(),
            target_path,
            link_type: LinkType::Wikilink,
            context,
            fragment,
        });
    }
}

/// Parse `[text](target)` markdown links from content.
fn parse_markdown_links(content: &str, source_path: &str, links: &mut Vec<ParsedLink>) {
    // Match [text](target) but not ![image](path) (images)
    let re = Regex::new(r"(?:^|[^!])\[([^\]\n]*?)]\(([^)\s\n]+?)\)")
        .expect("invalid markdown link regex");
    let source_dir = Path::new(source_path).parent();

    for cap in re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let target_raw = cap[2].trim();

        // Skip absolute URLs
        if target_raw.starts_with("http://")
            || target_raw.starts_with("https://")
            || target_raw.starts_with("mailto:")
        {
            continue;
        }

        // Skip pure fragment links (#heading)
        if target_raw.starts_with('#') {
            continue;
        }

        // Split on `#` for fragment
        let (path_part, fragment) = split_fragment(target_raw);
        if path_part.is_empty() {
            continue;
        }

        // Resolve relative path
        let target_path = resolve_markdown_target(&path_part, source_dir);

        let context = extract_context(content, full_match.start());

        links.push(ParsedLink {
            source_path: source_path.to_string(),
            raw_target: target_raw.to_string(),
            target_path,
            link_type: LinkType::Markdown,
            context,
            fragment,
        });
    }
}

/// Split a link target into (path, optional_fragment).
fn split_fragment(target: &str) -> (String, Option<String>) {
    if let Some(idx) = target.find('#') {
        let path = target[..idx].to_string();
        let fragment = target[idx + 1..].to_string();
        let frag = if fragment.is_empty() {
            None
        } else {
            Some(fragment)
        };
        (path, frag)
    } else {
        (target.to_string(), None)
    }
}

/// Resolve a wikilink target to a workspace-relative path.
///
/// Wikilinks use note names (stems) without extensions. If the target has no
/// extension, `.md` is appended. Paths are normalized with forward slashes.
fn resolve_wikilink_target(stem: &str, _source_dir: Option<&Path>) -> String {
    let normalized = stem.replace('\\', "/");
    let normalized = normalized.trim_start_matches("./");

    if Path::new(normalized).extension().is_none() {
        format!("{normalized}.md")
    } else {
        normalized.to_string()
    }
}

/// Resolve a markdown link target to a workspace-relative path.
///
/// Handles `./` prefix removal and normalizes separators.
fn resolve_markdown_target(target: &str, source_dir: Option<&Path>) -> String {
    let normalized = target.replace('\\', "/");
    let normalized = normalized.trim_start_matches("./");

    // If the target is relative (no leading slash), resolve against source directory
    if !normalized.starts_with('/') {
        if let Some(dir) = source_dir {
            if dir.as_os_str().is_empty() || dir == Path::new("") {
                return normalized.to_string();
            }
            let resolved = dir.join(normalized);
            return normalize_path(&resolved);
        }
    }

    normalized.to_string()
}

/// Normalize a path by collapsing `..` and `.` segments, using forward slashes.
fn normalize_path(path: &Path) -> String {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(s) => {
                components.push(s.to_string_lossy().into_owned());
            }
            _ => {}
        }
    }
    components.join("/")
}

/// Extract the context line containing a match, truncated to 200 characters.
fn extract_context(content: &str, byte_offset: usize) -> String {
    // Find the line containing this byte offset
    let before = &content[..byte_offset];
    let line_start = before.rfind('\n').map_or(0, |pos| pos + 1);

    let after = &content[byte_offset..];
    let line_end = after
        .find('\n')
        .map_or(content.len(), |pos| byte_offset + pos);

    let line = content[line_start..line_end].trim();

    if line.len() > 200 {
        format!("{}...", &line[..197])
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_wikilink() {
        let content = "See [[other-note]] for details.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "other-note.md");
        assert_eq!(links[0].link_type, LinkType::Wikilink);
        assert_eq!(links[0].raw_target, "other-note");
    }

    #[test]
    fn parse_wikilink_with_display_text() {
        let content = "See [[my-note|My Note Title]] here.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "my-note.md");
        assert_eq!(links[0].raw_target, "my-note");
    }

    #[test]
    fn parse_wikilink_with_fragment() {
        let content = "See [[other-note#section-1]] here.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "other-note.md");
        assert_eq!(links[0].fragment, Some("section-1".to_string()));
    }

    #[test]
    fn parse_wikilink_with_path() {
        let content = "See [[folder/deep-note]] here.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "folder/deep-note.md");
    }

    #[test]
    fn parse_wikilink_with_extension() {
        let content = "See [[report.pdf]] here.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "report.pdf");
    }

    #[test]
    fn parse_markdown_link() {
        let content = "Check [the docs](docs/readme.md) for info.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "docs/readme.md");
        assert_eq!(links[0].link_type, LinkType::Markdown);
    }

    #[test]
    fn parse_markdown_link_with_dot_slash() {
        let content = "See [note](./sibling.md) here.";
        let links = parse_links(content, "notes/source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "notes/sibling.md");
    }

    #[test]
    fn parse_markdown_link_with_fragment() {
        let content = "See [section](other.md#heading) here.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "other.md");
        assert_eq!(links[0].fragment, Some("heading".to_string()));
    }

    #[test]
    fn skip_http_links() {
        let content = "Visit [site](https://example.com) for info.";
        let links = parse_links(content, "source.md");
        assert!(links.is_empty());
    }

    #[test]
    fn skip_pure_fragment_links() {
        let content = "See [above](#top) for intro.";
        let links = parse_links(content, "source.md");
        assert!(links.is_empty());
    }

    #[test]
    fn skip_image_links() {
        let content = "Here is ![alt](image.png) an image.";
        let links = parse_links(content, "source.md");
        assert!(links.is_empty());
    }

    #[test]
    fn skip_links_in_code_blocks() {
        let content =
            "Before\n\n```\n[[should-skip]]\n[also](skip.md)\n```\n\nAfter [[real-link]]\n";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "real-link.md");
    }

    #[test]
    fn skip_links_in_fenced_code_with_language() {
        let content = "```rust\n[[not-a-link]]\n```\n\n[[actual-link]]\n";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "actual-link.md");
    }

    #[test]
    fn multiple_links_on_same_line() {
        let content = "See [[alpha]] and [[beta]] together.";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target_path, "alpha.md");
        assert_eq!(links[1].target_path, "beta.md");
    }

    #[test]
    fn mixed_link_types() {
        let content = "Wiki: [[wiki-note]]\nMarkdown: [md](other.md)\n";
        let links = parse_links(content, "source.md");
        assert_eq!(links.len(), 2);

        let wiki = links
            .iter()
            .find(|l| l.link_type == LinkType::Wikilink)
            .unwrap();
        assert_eq!(wiki.target_path, "wiki-note.md");

        let md = links
            .iter()
            .find(|l| l.link_type == LinkType::Markdown)
            .unwrap();
        assert_eq!(md.target_path, "other.md");
    }

    #[test]
    fn context_is_truncated_to_200_chars() {
        let long_line = format!("{}[[target]]{}", "a".repeat(150), "b".repeat(100));
        let links = parse_links(&long_line, "source.md");
        assert_eq!(links.len(), 1);
        assert!(links[0].context.len() <= 200);
    }

    #[test]
    fn resolve_relative_markdown_link_from_subdirectory() {
        let content = "See [note](../sibling.md) here.";
        let links = parse_links(content, "sub/deep/source.md");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_path, "sub/sibling.md");
    }

    #[test]
    fn empty_content_returns_no_links() {
        let links = parse_links("", "source.md");
        assert!(links.is_empty());
    }

    #[test]
    fn no_links_in_plain_text() {
        let links = parse_links("Just some plain text without any links.", "source.md");
        assert!(links.is_empty());
    }

    #[test]
    fn split_fragment_works() {
        let (path, frag) = split_fragment("note#heading");
        assert_eq!(path, "note");
        assert_eq!(frag, Some("heading".to_string()));

        let (path, frag) = split_fragment("note");
        assert_eq!(path, "note");
        assert_eq!(frag, None);

        let (path, frag) = split_fragment("note#");
        assert_eq!(path, "note");
        assert_eq!(frag, None);
    }
}

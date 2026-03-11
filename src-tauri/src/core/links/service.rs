//! In-memory link graph with forward and reverse indices.
//!
//! `LinkGraph` maintains two hashmaps:
//! - **Outgoing links**: `HashMap<String, Vec<LinkEdge>>` keyed by source path
//! - **Backlinks**: `HashMap<String, Vec<BacklinkRecord>>` keyed by target path
//!
//! The graph is rebuilt from the [`FileIndex`] after each scan. It supports
//! querying backlinks, outgoing links, and the full graph for visualization.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::index::FileIndex;
use crate::core::links::parser;
#[cfg(test)]
use crate::core::links::parser::ParsedLink;
use crate::models::links::{BacklinkRecord, LinkEdge, NoteGraphResponse, NoteNode};

/// Stable node ID derived from workspace-relative path.
fn node_id(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Thread-safe in-memory link graph.
///
/// All public methods acquire the internal `RwLock` as needed; callers do not
/// need external synchronization.
#[derive(Debug, Clone)]
pub struct LinkGraph {
    inner: Arc<RwLock<GraphInner>>,
}

#[derive(Debug, Default)]
struct GraphInner {
    /// Outgoing link edges keyed by source workspace-relative path.
    outgoing: HashMap<String, Vec<LinkEdge>>,
    /// Backlink records keyed by target workspace-relative path.
    backlinks: HashMap<String, Vec<BacklinkRecord>>,
    /// All known note paths (from the index).
    known_notes: HashSet<String>,
}

impl LinkGraph {
    /// Create a new empty link graph.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GraphInner::default())),
        }
    }

    /// Rebuild the entire link graph from the current file index state.
    ///
    /// This reads all notes from the index, parses their content for links,
    /// and builds both the forward and reverse indices.
    pub fn rebuild(&self, index: &FileIndex) {
        let all_notes = index.get_all_notes();
        let root = index.root().to_path_buf();

        // Collect the set of known note paths for resolution
        let known_notes: HashSet<String> = all_notes.iter().map(|n| n.path.clone()).collect();

        let mut outgoing: HashMap<String, Vec<LinkEdge>> = HashMap::new();
        let mut backlinks: HashMap<String, Vec<BacklinkRecord>> = HashMap::new();

        for note in &all_notes {
            // Read the file content to parse links with full context
            let abs_path = root.join(&note.path);
            let content = match std::fs::read_to_string(&abs_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let parsed = parser::parse_links(&content, &note.path);

            let mut edges = Vec::new();
            for link in &parsed {
                let from_id = node_id(&note.path);
                let to_id = node_id(&link.target_path);

                let edge = LinkEdge {
                    from_node_id: from_id,
                    to_node_id: to_id,
                    kind: link.link_type,
                };
                edges.push(edge);

                // Build the backlink record
                let record = BacklinkRecord {
                    source_path: note.path.clone(),
                    target_path: link.target_path.clone(),
                    context_snippet: link.context.clone(),
                };

                backlinks
                    .entry(link.target_path.clone())
                    .or_default()
                    .push(record);
            }

            if !edges.is_empty() {
                outgoing.insert(note.path.clone(), edges);
            }
        }

        // Replace the inner graph atomically
        let mut inner = self.inner.write();
        inner.outgoing = outgoing;
        inner.backlinks = backlinks;
        inner.known_notes = known_notes;
    }

    /// Rebuild from parsed links directly (for testing without filesystem).
    #[cfg(test)]
    pub fn rebuild_from_parsed(
        &self,
        all_links: &[ParsedLink],
        known_notes: HashSet<String>,
    ) {
        let mut outgoing: HashMap<String, Vec<LinkEdge>> = HashMap::new();
        let mut backlinks: HashMap<String, Vec<BacklinkRecord>> = HashMap::new();

        for link in all_links {
            let from_id = node_id(&link.source_path);
            let to_id = node_id(&link.target_path);

            let edge = LinkEdge {
                from_node_id: from_id,
                to_node_id: to_id,
                kind: link.link_type,
            };

            outgoing
                .entry(link.source_path.clone())
                .or_default()
                .push(edge);

            let record = BacklinkRecord {
                source_path: link.source_path.clone(),
                target_path: link.target_path.clone(),
                context_snippet: link.context.clone(),
            };

            backlinks
                .entry(link.target_path.clone())
                .or_default()
                .push(record);
        }

        let mut inner = self.inner.write();
        inner.outgoing = outgoing;
        inner.backlinks = backlinks;
        inner.known_notes = known_notes;
    }

    /// Get all backlinks for a given workspace-relative path.
    ///
    /// Returns an empty vec if no notes link to the target.
    pub fn get_backlinks(&self, path: &str) -> Vec<BacklinkRecord> {
        let inner = self.inner.read();
        inner
            .backlinks
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all outgoing link edges for a given workspace-relative path.
    ///
    /// Returns an empty vec if the note has no outgoing links.
    pub fn get_outgoing_links(&self, path: &str) -> Vec<LinkEdge> {
        let inner = self.inner.read();
        inner
            .outgoing
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    /// Get the full graph for visualization.
    ///
    /// Builds `NoteNode` entries for all known notes and collects all edges.
    /// Notes that are linked to but don't exist are included with the path as
    /// title (dangling links - they still appear in the graph).
    pub fn get_graph(&self, index: &FileIndex) -> NoteGraphResponse {
        let inner = self.inner.read();
        let all_notes = index.get_all_notes();

        let mut nodes: Vec<NoteNode> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        // Add all known notes as nodes
        for note in &all_notes {
            let id = node_id(&note.path);
            if seen_ids.insert(id.clone()) {
                nodes.push(NoteNode {
                    id,
                    path: note.path.clone(),
                    title: note.title.clone(),
                    tags: note.tags.clone(),
                });
            }
        }

        // Add dangling link targets as nodes (resolved=false conceptually)
        for targets in inner.backlinks.keys() {
            let id = node_id(targets);
            if seen_ids.insert(id.clone()) {
                // This target doesn't exist as a note - create a placeholder node
                let title = std::path::Path::new(targets)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| targets.to_string());
                nodes.push(NoteNode {
                    id,
                    path: targets.clone(),
                    title,
                    tags: Vec::new(),
                });
            }
        }

        // Collect all edges
        let edges: Vec<LinkEdge> = inner
            .outgoing
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect();

        NoteGraphResponse { nodes, edges }
    }

    /// Check if the graph has been populated.
    pub fn is_populated(&self) -> bool {
        let inner = self.inner.read();
        !inner.known_notes.is_empty()
    }
}

impl Default for LinkGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::links::LinkType;

    fn make_parsed_link(source: &str, target: &str, kind: LinkType, context: &str) -> ParsedLink {
        ParsedLink {
            source_path: source.to_string(),
            raw_target: target.to_string(),
            target_path: if std::path::Path::new(target).extension().is_none() {
                format!("{target}.md")
            } else {
                target.to_string()
            },
            link_type: kind,
            context: context.to_string(),
            fragment: None,
        }
    }

    #[test]
    fn new_graph_is_empty() {
        let graph = LinkGraph::new();
        assert!(!graph.is_populated());
        assert!(graph.get_backlinks("any.md").is_empty());
        assert!(graph.get_outgoing_links("any.md").is_empty());
    }

    #[test]
    fn backlinks_are_indexed() {
        let graph = LinkGraph::new();
        let links = vec![
            make_parsed_link("a.md", "target.md", LinkType::Wikilink, "See [[target]]"),
            make_parsed_link("b.md", "target.md", LinkType::Markdown, "[link](target.md)"),
        ];

        let mut known = HashSet::new();
        known.insert("a.md".to_string());
        known.insert("b.md".to_string());
        known.insert("target.md".to_string());
        graph.rebuild_from_parsed(&links, known);

        let backlinks = graph.get_backlinks("target.md");
        assert_eq!(backlinks.len(), 2);

        let sources: Vec<&str> = backlinks.iter().map(|b| b.source_path.as_str()).collect();
        assert!(sources.contains(&"a.md"));
        assert!(sources.contains(&"b.md"));
    }

    #[test]
    fn outgoing_links_are_indexed() {
        let graph = LinkGraph::new();
        let links = vec![
            make_parsed_link("source.md", "alpha.md", LinkType::Wikilink, "[[alpha]]"),
            make_parsed_link("source.md", "beta.md", LinkType::Markdown, "[b](beta.md)"),
        ];

        let mut known = HashSet::new();
        known.insert("source.md".to_string());
        known.insert("alpha.md".to_string());
        known.insert("beta.md".to_string());
        graph.rebuild_from_parsed(&links, known);

        let outgoing = graph.get_outgoing_links("source.md");
        assert_eq!(outgoing.len(), 2);
    }

    #[test]
    fn dangling_links_are_preserved() {
        let graph = LinkGraph::new();
        let links = vec![make_parsed_link(
            "source.md",
            "nonexistent.md",
            LinkType::Wikilink,
            "[[nonexistent]]",
        )];

        let mut known = HashSet::new();
        known.insert("source.md".to_string());
        // Note: nonexistent.md is NOT in known_notes
        graph.rebuild_from_parsed(&links, known);

        // Backlinks for the dangling target should still exist
        let backlinks = graph.get_backlinks("nonexistent.md");
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_path, "source.md");
    }

    #[test]
    fn link_type_is_preserved() {
        let graph = LinkGraph::new();
        let links = vec![
            make_parsed_link("a.md", "wiki-target.md", LinkType::Wikilink, "[[wiki-target]]"),
            make_parsed_link("a.md", "md-target.md", LinkType::Markdown, "[t](md-target.md)"),
        ];

        let mut known = HashSet::new();
        known.insert("a.md".to_string());
        graph.rebuild_from_parsed(&links, known);

        let outgoing = graph.get_outgoing_links("a.md");
        assert_eq!(outgoing.len(), 2);

        let wiki = outgoing.iter().find(|e| e.kind == LinkType::Wikilink).unwrap();
        assert_eq!(wiki.to_node_id, node_id("wiki-target.md"));

        let md = outgoing.iter().find(|e| e.kind == LinkType::Markdown).unwrap();
        assert_eq!(md.to_node_id, node_id("md-target.md"));
    }

    #[test]
    fn node_id_is_deterministic() {
        assert_eq!(node_id("foo.md"), node_id("foo.md"));
        assert_ne!(node_id("foo.md"), node_id("bar.md"));
    }
}

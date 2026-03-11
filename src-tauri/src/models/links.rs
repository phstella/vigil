//! Link graph, backlink, and note-node models for bidirectional linking.

use serde::{Deserialize, Serialize};

/// Link syntax type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// `[[target]]` or `[[target|display text]]`
    Wikilink,
    /// `[text](relative/path.md)` or `[text](./path.md)`
    Markdown,
}

/// A node in the bidirectional link graph, representing one note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteNode {
    /// Stable hash of workspace-relative path.
    pub id: String,
    /// Workspace-relative path.
    pub path: String,
    /// Note title (see NoteMetadata title extraction).
    pub title: String,
    /// Tags on this note.
    pub tags: Vec<String>,
}

/// A directed edge in the link graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEdge {
    /// Source note node ID.
    pub from_node_id: String,
    /// Target note node ID.
    pub to_node_id: String,
    /// Link syntax type.
    pub kind: LinkType,
}

/// A single backlink entry with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklinkRecord {
    /// Workspace-relative path of the note containing the link.
    pub source_path: String,
    /// Workspace-relative path of the linked note.
    pub target_path: String,
    /// The line containing the link (truncated to 200 chars).
    pub context_snippet: String,
}

/// Response payload for the `get_backlinks` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklinksResponse {
    /// Notes linking to this file.
    pub backlinks: Vec<BacklinkRecord>,
}

/// Response payload for the `get_note_graph` command (Epic 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteGraphResponse {
    /// All notes as graph nodes.
    pub nodes: Vec<NoteNode>,
    /// All links as graph edges.
    pub edges: Vec<LinkEdge>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_type_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&LinkType::Wikilink).unwrap(),
            "\"wikilink\""
        );
        assert_eq!(
            serde_json::to_string(&LinkType::Markdown).unwrap(),
            "\"markdown\""
        );
    }

    #[test]
    fn note_node_roundtrip() {
        let node = NoteNode {
            id: "hash123".into(),
            path: "notes/hello.md".into(),
            title: "Hello World".into(),
            tags: vec!["greeting".into()],
        };
        let json = serde_json::to_string(&node).unwrap();
        let deser: NoteNode = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.id, "hash123");
        assert_eq!(deser.tags, vec!["greeting"]);
    }

    #[test]
    fn link_edge_roundtrip() {
        let edge = LinkEdge {
            from_node_id: "a".into(),
            to_node_id: "b".into(),
            kind: LinkType::Wikilink,
        };
        let json = serde_json::to_string(&edge).unwrap();
        let deser: LinkEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.kind, LinkType::Wikilink);
    }

    #[test]
    fn backlink_record_roundtrip() {
        let rec = BacklinkRecord {
            source_path: "journal/2024.md".into(),
            target_path: "projects/vigil.md".into(),
            context_snippet: "Working on [[vigil]] today".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let deser: BacklinkRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.source_path, "journal/2024.md");
    }
}

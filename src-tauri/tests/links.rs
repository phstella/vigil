//! Integration tests for the link graph and backlinks resolver.

use std::collections::HashSet;
use std::fs;

use vigil_lib::core::index::FileIndex;
use vigil_lib::core::links::parser::parse_links;
use vigil_lib::core::links::LinkGraph;
use vigil_lib::models::links::LinkType;

fn temp_workspace() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

// ---------------------------------------------------------------------------
// Parser tests
// ---------------------------------------------------------------------------

#[test]
fn parser_extracts_wikilinks() {
    let content = "# My Note\n\nSee [[other-note]] and [[folder/deep|Display Name]].\n";
    let links = parse_links(content, "source.md");

    assert_eq!(links.len(), 2);

    let first = &links[0];
    assert_eq!(first.target_path, "other-note.md");
    assert_eq!(first.link_type, LinkType::Wikilink);
    assert_eq!(first.source_path, "source.md");

    let second = &links[1];
    assert_eq!(second.target_path, "folder/deep.md");
    assert_eq!(second.link_type, LinkType::Wikilink);
}

#[test]
fn parser_extracts_markdown_links() {
    let content = "Check [docs](./docs/readme.md) and [other](../sibling.md).\n";
    let links = parse_links(content, "notes/source.md");

    assert_eq!(links.len(), 2);

    let first = &links[0];
    assert_eq!(first.target_path, "notes/docs/readme.md");
    assert_eq!(first.link_type, LinkType::Markdown);

    let second = &links[1];
    assert_eq!(second.target_path, "sibling.md");
    assert_eq!(second.link_type, LinkType::Markdown);
}

#[test]
fn parser_skips_code_blocks() {
    let content = "Real [[link-a]]\n\n```\n[[fake-link]]\n```\n\nReal [[link-b]]\n";
    let links = parse_links(content, "source.md");

    let targets: Vec<&str> = links.iter().map(|l| l.target_path.as_str()).collect();
    assert!(targets.contains(&"link-a.md"));
    assert!(targets.contains(&"link-b.md"));
    assert!(!targets.contains(&"fake-link.md"));
}

#[test]
fn parser_handles_fragments() {
    let content = "See [[note#section]] and [link](other.md#heading).\n";
    let links = parse_links(content, "source.md");

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].target_path, "note.md");
    assert_eq!(links[0].fragment, Some("section".to_string()));
    assert_eq!(links[1].target_path, "other.md");
    assert_eq!(links[1].fragment, Some("heading".to_string()));
}

#[test]
fn parser_skips_external_urls() {
    let content = "See [site](https://example.com) and [local](note.md).\n";
    let links = parse_links(content, "source.md");

    assert_eq!(links.len(), 1);
    assert_eq!(links[0].target_path, "note.md");
}

#[test]
fn parser_preserves_link_type_distinction() {
    let content = "Wiki: [[wiki-note]]\nMarkdown: [text](md-note.md)\n";
    let links = parse_links(content, "source.md");

    assert_eq!(links.len(), 2);
    let wiki = links.iter().find(|l| l.link_type == LinkType::Wikilink).unwrap();
    assert_eq!(wiki.target_path, "wiki-note.md");

    let md = links.iter().find(|l| l.link_type == LinkType::Markdown).unwrap();
    assert_eq!(md.target_path, "md-note.md");
}

// ---------------------------------------------------------------------------
// LinkGraph integration tests with real filesystem
// ---------------------------------------------------------------------------

#[test]
fn link_graph_rebuild_from_index() {
    let dir = temp_workspace();

    // Create notes that link to each other
    fs::write(
        dir.path().join("a.md"),
        "# Note A\n\nLinks to [[b]] and [c](c.md).\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("b.md"),
        "# Note B\n\nLinks back to [[a]].\n",
    )
    .unwrap();

    fs::write(dir.path().join("c.md"), "# Note C\n\nNo outgoing links.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    // a.md has outgoing links to b.md and c.md
    let a_out = graph.get_outgoing_links("a.md");
    assert_eq!(a_out.len(), 2);

    // b.md has outgoing link to a.md
    let b_out = graph.get_outgoing_links("b.md");
    assert_eq!(b_out.len(), 1);

    // c.md has no outgoing links
    let c_out = graph.get_outgoing_links("c.md");
    assert!(c_out.is_empty());

    // Backlinks: b.md is linked from a.md
    let b_backlinks = graph.get_backlinks("b.md");
    assert_eq!(b_backlinks.len(), 1);
    assert_eq!(b_backlinks[0].source_path, "a.md");

    // Backlinks: a.md is linked from b.md
    let a_backlinks = graph.get_backlinks("a.md");
    assert_eq!(a_backlinks.len(), 1);
    assert_eq!(a_backlinks[0].source_path, "b.md");

    // Backlinks: c.md is linked from a.md
    let c_backlinks = graph.get_backlinks("c.md");
    assert_eq!(c_backlinks.len(), 1);
    assert_eq!(c_backlinks[0].source_path, "a.md");
}

#[test]
fn link_graph_includes_dangling_links() {
    let dir = temp_workspace();

    // Create a note that links to a non-existent note
    fs::write(
        dir.path().join("source.md"),
        "# Source\n\nSee [[nonexistent]].\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    // The dangling link should still produce a backlink record
    let backlinks = graph.get_backlinks("nonexistent.md");
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].source_path, "source.md");

    // The graph visualization should include a node for the dangling target
    let note_graph = graph.get_graph(&index);
    let paths: HashSet<&str> = note_graph.nodes.iter().map(|n| n.path.as_str()).collect();
    assert!(paths.contains("source.md"));
    assert!(paths.contains("nonexistent.md"));
}

#[test]
fn link_graph_get_graph_returns_all_nodes_and_edges() {
    let dir = temp_workspace();

    fs::write(
        dir.path().join("a.md"),
        "# Note A\n\nLinks to [[b]].\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("b.md"),
        "# Note B\n\nLinks to [[c]].\n",
    )
    .unwrap();
    fs::write(dir.path().join("c.md"), "# Note C\n\nNo links.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    let note_graph = graph.get_graph(&index);

    assert_eq!(note_graph.nodes.len(), 3);
    assert_eq!(note_graph.edges.len(), 2);

    // Verify node titles come from the index
    let a_node = note_graph.nodes.iter().find(|n| n.path == "a.md").unwrap();
    assert_eq!(a_node.title, "Note A");
}

#[test]
fn link_graph_context_snippet_is_present() {
    let dir = temp_workspace();

    fs::write(
        dir.path().join("source.md"),
        "# Source\n\nThis line links to [[target]] with context.\n",
    )
    .unwrap();
    fs::write(dir.path().join("target.md"), "# Target\n\nContent.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    let backlinks = graph.get_backlinks("target.md");
    assert_eq!(backlinks.len(), 1);
    assert!(backlinks[0].context_snippet.contains("[[target]]"));
}

#[test]
fn link_graph_handles_empty_workspace() {
    let dir = temp_workspace();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    assert!(!graph.is_populated());
    assert!(graph.get_backlinks("any.md").is_empty());

    let note_graph = graph.get_graph(&index);
    assert!(note_graph.nodes.is_empty());
    assert!(note_graph.edges.is_empty());
}

#[test]
fn link_graph_rebuild_replaces_stale_data() {
    let dir = temp_workspace();

    fs::write(
        dir.path().join("a.md"),
        "# A\n\nLinks to [[b]].\n",
    )
    .unwrap();
    fs::write(dir.path().join("b.md"), "# B\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    assert_eq!(graph.get_backlinks("b.md").len(), 1);

    // Modify the file to remove the link
    fs::write(dir.path().join("a.md"), "# A\n\nNo more links.\n").unwrap();

    // Rebuild index and graph
    index.full_scan();
    graph.rebuild(&index);

    // Stale backlink should be gone
    assert_eq!(graph.get_backlinks("b.md").len(), 0);
}

#[test]
fn link_graph_handles_subdirectory_links() {
    let dir = temp_workspace();

    fs::create_dir(dir.path().join("sub")).unwrap();

    fs::write(
        dir.path().join("root.md"),
        "# Root\n\nSee [[sub/deep]].\n",
    )
    .unwrap();
    fs::write(dir.path().join("sub/deep.md"), "# Deep\n\nContent.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    let backlinks = graph.get_backlinks("sub/deep.md");
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].source_path, "root.md");
}

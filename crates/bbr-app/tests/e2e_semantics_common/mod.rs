use bbr_testkit::protocol::{SemanticNode, SemanticRole};

// Shared helper module included by several E2E binaries; each binary uses a
// subset of the helpers, so dead-code is expected for the unused rest.
#[allow(dead_code)]
pub const SIDEBAR_DEFAULT: f32 = 280.0;

pub fn find<'a>(nodes: &'a [SemanticNode], id: &str) -> Option<&'a SemanticNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let Some(found) = find(&node.children, id) {
            return Some(found);
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn assert_node(
    tree: &[SemanticNode],
    id: &str,
    role: SemanticRole,
    name: Option<&str>,
    value: Option<&str>,
    enabled: bool,
    selected: bool,
    focused: bool,
) -> SemanticNode {
    let node = find(tree, id)
        .unwrap_or_else(|| panic!("semantic tree missing {id}; ids: {:?}", node_ids(tree)));
    assert_eq!(node.role, role, "{id} role");
    assert_eq!(node.name.as_deref(), name, "{id} name");
    assert_eq!(node.value.as_deref(), value, "{id} value");
    assert_eq!(node.enabled, enabled, "{id} enabled");
    assert_eq!(node.selected, selected, "{id} selected");
    assert_eq!(node.focused, focused, "{id} focused");
    node.clone()
}

#[allow(dead_code)]
fn node_ids(tree: &[SemanticNode]) -> Vec<String> {
    let mut out = Vec::new();
    for node in tree {
        out.push(node.id.clone());
        out.extend(node_ids(&node.children));
    }
    out
}

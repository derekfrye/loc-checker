use tree_sitter::Node;

pub(super) fn callable_name(node: Node<'_>, source: &[u8]) -> String {
    match node.kind() {
        "constructor_declaration"
        | "method_declaration"
        | "property_declaration"
        | "event_declaration" => {
            name_for_node(node, source)
        }
        "destructor_declaration" => format!("~{}", name_for_node(node, source)),
        "indexer_declaration" => "this[]".to_string(),
        "operator_declaration" => operator_name(node, source),
        "conversion_operator_declaration" => conversion_operator_name(node, source),
        _ => name_for_node(node, source),
    }
}

pub(super) fn accessor_name(node: Node<'_>, source: &[u8], parent_name: &str) -> String {
    let accessor = child_field_text(node, source, "name").unwrap_or("accessor");
    format!("{parent_name}.{accessor}")
}

pub(super) fn child_field_text<'a>(
    node: Node<'_>,
    source: &'a [u8],
    field: &str,
) -> Option<&'a str> {
    node.child_by_field_name(field).and_then(|child| text(child, source))
}

pub(super) fn first_identifier(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find_map(|child| {
        if child.kind() == "identifier" {
            text(child, source).map(std::string::ToString::to_string)
        } else {
            None
        }
    })
}

pub(super) fn has_modifier(node: Node<'_>, source: &[u8], expected: &str) -> bool {
    node.children(&mut node.walk())
        .any(|child| child.kind() == "modifier" && text(child, source) == Some(expected))
}

pub(super) fn name_for_node(node: Node<'_>, source: &[u8]) -> String {
    if let Some(name_node) = node.child_by_field_name("name")
        && let Some(value) = text(name_node, source)
    {
        return value.to_string();
    }
    if let Some(value) = first_declarator_name(node, source) {
        return value;
    }
    first_identifier(node, source).unwrap_or_else(|| node.kind().to_string())
}

fn text<'a>(node: Node<'_>, source: &'a [u8]) -> Option<&'a str> {
    node.utf8_text(source).ok()
}

fn conversion_operator_name(node: Node<'_>, source: &[u8]) -> String {
    let modifier = if has_modifier(node, source, "implicit") {
        "implicit"
    } else {
        "explicit"
    };
    let target = child_field_text(node, source, "type").unwrap_or("unknown");
    format!("{modifier} operator {target}")
}

fn operator_name(node: Node<'_>, source: &[u8]) -> String {
    let operator = child_field_text(node, source, "operator").unwrap_or("operator");
    format!("operator {operator}")
}

fn first_declarator_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find_map(|child| {
        if child.kind() == "variable_declaration" {
            let mut inner = child.walk();
            child.children(&mut inner).find_map(|grandchild| {
                if grandchild.kind() == "variable_declarator" {
                    child_field_text(grandchild, source, "name")
                        .map(std::string::ToString::to_string)
                } else {
                    None
                }
            })
        } else if child.kind() == "variable_declarator" {
            child_field_text(child, source, "name").map(std::string::ToString::to_string)
        } else {
            None
        }
    })
}

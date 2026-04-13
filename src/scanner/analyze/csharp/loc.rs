use tree_sitter::Node;

pub(super) struct LineCounter<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineCounter<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
        }
    }

    pub(super) fn record(&self, node: Node<'_>) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }

        let start = node.start_position().row;
        let mut end = node.end_position().row;
        if end >= self.lines.len() {
            end = self.lines.len().saturating_sub(1);
        }
        if end < start {
            return None;
        }

        let count = self.lines[start..=end]
            .iter()
            .filter(|line| !line.trim().is_empty())
            .count();
        if count == 0 { None } else { Some(count) }
    }
}

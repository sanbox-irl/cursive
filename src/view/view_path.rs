/// Represents a path to a single view in the layout.
pub struct ViewPath<'a> {
    /// List of turns to make on decision nodes when descending the view tree.
    /// Simple nodes (with one fixed child) are skipped.
    pub path: stack_list::Node<'a, usize>,
}

impl Default for ViewPath<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ViewPath<'a> {
    /// Creates a new empty path.
    pub fn new() -> Self {
        ViewPath {
            path: stack_list::Node::new(),
        }
    }

    /// Creates a new `ViewPath` with an extra step.
    pub fn add(&'a self, n: usize) -> Self {
        ViewPath {
            path: self.path.prepend(n),
        }
    }
}

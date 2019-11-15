use crate::view::ViewPath;

/// Selects a single view (if any) in the tree.
pub enum Selector<'a> {
    /// Selects a view from its ID.
    Id(&'a str),
    /// Selects a view from its path.
    Path(&'a ViewPath<'a>),
}

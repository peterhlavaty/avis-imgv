//! The shape of a history: an arena of nodes with one cursor.
//!
//! A list would do if going back always meant forgetting. It does not here —
//! nothing is ever overwritten — so going back and then doing something
//! different has to keep both halves, and what that makes is a tree. The
//! branch just left is still there, still complete, and still reachable from
//! the panel.
//!
//! Nodes live in a `Vec` and are addressed by index rather than by pointer,
//! which is what lets the panel hold on to an identifier across a trim. A slot
//! whose node has been dropped is left as `None` rather than reused, so an
//! index never quietly comes to mean a different node.
//!
//! This is generic over what a node carries so that its own tests are about
//! the shape — a branch, a route, a trim — and can be read without knowing
//! anything about deeds or photographs.

use super::files::Way;

/// Where a node is. Stable for the life of the tree.
pub type NodeId = usize;

/// One node and its place among the others.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Node<T> {
    /// Where this came from. Only the root has none.
    pub parent: Option<NodeId>,
    /// What was done after this, in the order it was first done.
    pub children: Vec<NodeId>,
    /// The child a redo takes, which is the one most recently gone down.
    ///
    /// Without this, coming back to a fork and pressing redo would take the
    /// newest branch rather than the one just left, which is the opposite of
    /// what somebody retracing their steps means by it.
    pub preferred: Option<NodeId>,
    /// What the node holds.
    pub value: T,
}

/// A history and the one place in it that is *now*.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Tree<T> {
    nodes: Vec<Option<Node<T>>>,
    root: NodeId,
    cursor: NodeId,
}

impl<T> Tree<T> {
    /// A tree holding nothing but the beginning.
    ///
    /// The root is a node like any other so that every real node has a parent
    /// and the walking code needs no special case for the first one. Its value
    /// is never run.
    pub fn new(root: T) -> Tree<T> {
        Tree {
            nodes: vec![Some(Node {
                parent: None,
                children: Vec::new(),
                preferred: None,
                value: root,
            })],
            root: 0,
            cursor: 0,
        }
    }

    /// The beginning, which is never run and never trimmed.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Where the history is now.
    pub fn cursor(&self) -> NodeId {
        self.cursor
    }

    /// Moves to a node without running anything.
    ///
    /// The preferred child of everything on the way is set to the way taken,
    /// so a redo from further back retraces this route rather than guessing.
    pub fn set_cursor(&mut self, to: NodeId) {
        if self.get(to).is_none() {
            return;
        }

        let mut at = to;
        while let Some(parent) = self.get(at).and_then(|node| node.parent) {
            if let Some(node) = self.nodes.get_mut(parent).and_then(Option::as_mut) {
                node.preferred = Some(at);
            }
            at = parent;
        }

        self.cursor = to;
    }

    /// Adds a node under the cursor and moves onto it.
    ///
    /// This is what makes a branch: done after going back, the new node joins
    /// the node the cursor is on and the one that was there before stays where
    /// it is, whole.
    pub fn push(&mut self, value: T) -> NodeId {
        let id = self.nodes.len();
        let parent = self.cursor;

        self.nodes.push(Some(Node {
            parent: Some(parent),
            children: Vec::new(),
            preferred: None,
            value,
        }));

        if let Some(node) = self.nodes.get_mut(parent).and_then(Option::as_mut) {
            node.children.push(id);
            node.preferred = Some(id);
        }

        self.cursor = id;
        id
    }

    /// The node at an index, if it is still there.
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id).and_then(Option::as_ref)
    }

    /// What the node holds, if it is still there.
    pub fn value(&self, id: NodeId) -> Option<&T> {
        self.get(id).map(|node| &node.value)
    }

    /// What the node holds, to be changed in place.
    ///
    /// Only for folding one row into the one it continues; the shape of the
    /// tree is never edited through this.
    pub fn value_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.nodes
            .get_mut(id)
            .and_then(Option::as_mut)
            .map(|node| &mut node.value)
    }

    /// Every node still in the tree, in the order the nodes were made.
    ///
    /// Creation order rather than tree order, because that is the order the
    /// things actually happened in, which is what the panel is a list of.
    pub fn in_order(&self) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(id, node)| node.as_ref().map(|node| (id, node)))
    }

    /// How many nodes there are, not counting the beginning.
    pub fn len(&self) -> usize {
        self.nodes.iter().filter(|node| node.is_some()).count() - 1
    }

    /// Whether nothing has been done yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The node an undo would run, which is the one the cursor is on.
    pub fn to_undo(&self) -> Option<NodeId> {
        (self.cursor != self.root).then_some(self.cursor)
    }

    /// The node a redo would run, which is the child last gone down.
    pub fn to_redo(&self) -> Option<NodeId> {
        let node = self.get(self.cursor)?;

        node.preferred
            .filter(|id| self.get(*id).is_some())
            .or_else(|| node.children.last().copied())
    }

    /// How deep a node is, for drawing a branch as an indent.
    pub fn depth(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut at = id;

        while let Some(parent) = self.get(at).and_then(|node| node.parent) {
            depth += 1;
            at = parent;
        }

        depth
    }

    /// How many forks lie between the beginning and this node.
    ///
    /// Not [`Tree::depth`], which counts every step and so would indent a
    /// straight run of two hundred deeds two hundred times over. What a reader
    /// wants to see is where the history *branched*: a node is one level in
    /// for every ancestor it reaches by a way that was not the first way tried
    /// from there.
    pub fn branch_depth(&self, id: NodeId) -> usize {
        let mut forks = 0;
        let mut at = id;

        while let Some(node) = self.get(at) {
            let Some(parent) = node.parent else { break };

            if self
                .get(parent)
                .is_some_and(|parent| parent.children.first() != Some(&at))
            {
                forks += 1;
            }

            at = parent;
        }

        forks
    }

    /// Whether `elder` is on the way from the beginning to `id`.
    pub fn is_ancestor(&self, elder: NodeId, id: NodeId) -> bool {
        let mut at = Some(id);

        while let Some(node) = at {
            if node == elder {
                return true;
            }
            at = self.get(node).and_then(|node| node.parent);
        }

        false
    }

    /// Every node from the one given back to the beginning, nearest first.
    fn ancestry(&self, from: NodeId) -> Vec<NodeId> {
        let mut line = Vec::new();
        let mut at = Some(from);

        while let Some(id) = at {
            line.push(id);
            at = self.get(id).and_then(|node| node.parent);
        }

        line
    }

    /// What running the history from where it is to the node given would do.
    ///
    /// Up to the nearest node the two have in common, running each deed
    /// backwards; then down the other side, running each forwards. Going to a
    /// node on the same line is the ordinary undo or redo, several at a time,
    /// and the general case is what a click in the panel means.
    pub fn route(&self, to: NodeId) -> Vec<(NodeId, Way)> {
        if self.get(to).is_none() || to == self.cursor {
            return Vec::new();
        }

        let up = self.ancestry(self.cursor);
        let down = self.ancestry(to);

        let Some(meeting) = up.iter().find(|id| down.contains(id)).copied() else {
            return Vec::new();
        };

        let mut route: Vec<(NodeId, Way)> = up
            .iter()
            .take_while(|id| **id != meeting)
            .map(|id| (*id, Way::Back))
            .collect();

        let descent: Vec<NodeId> = down
            .iter()
            .take_while(|id| **id != meeting)
            .copied()
            .collect();

        route.extend(descent.into_iter().rev().map(|id| (id, Way::Forward)));

        route
    }

    /// Forgets the oldest nodes until no more than `keep` are left.
    ///
    /// The oldest rather than the furthest from the cursor, because "the last
    /// two hundred things I did" is what a limit on a history means. A dropped
    /// node's children are given to its parent, so a branch hanging off it
    /// survives losing the step that made it — what goes is that one deed, not
    /// everything after it.
    ///
    /// Nothing on the way from the beginning to the cursor by way of the
    /// cursor itself is dropped while it is still needed to get back, so a
    /// small limit shortens the history rather than corrupting it.
    pub fn trim(&mut self, keep: usize) {
        if keep == 0 {
            return;
        }

        while self.len() > keep {
            let Some(oldest) = self.oldest_droppable() else {
                return;
            };

            self.drop_node(oldest);
        }
    }

    /// The oldest node that may go: not the beginning, and not where we are.
    fn oldest_droppable(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .find(|(id, node)| *id != self.root && *id != self.cursor && node.is_some())
            .map(|(id, _)| id)
    }

    /// Takes one node out, handing its children to its parent.
    fn drop_node(&mut self, id: NodeId) {
        let Some(node) = self.nodes.get(id).and_then(Option::as_ref) else {
            return;
        };

        let Some(parent) = node.parent else {
            return;
        };

        let children = node.children.clone();

        for child in &children {
            if let Some(child) = self.nodes.get_mut(*child).and_then(Option::as_mut) {
                child.parent = Some(parent);
            }
        }

        if let Some(parent) = self.nodes.get_mut(parent).and_then(Option::as_mut) {
            let at = parent.children.iter().position(|child| *child == id);

            if let Some(at) = at {
                parent.children.splice(at..=at, children);
            }

            if parent.preferred == Some(id) {
                parent.preferred = parent.children.last().copied();
            }
        }

        self.nodes[id] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tree of `&str`, so the assertions read as the shape they are about.
    fn tree() -> Tree<&'static str> {
        Tree::new("start")
    }

    #[test]
    fn a_new_tree_is_at_the_beginning_with_nothing_to_do() {
        let tree = tree();

        assert!(tree.is_empty());
        assert_eq!(tree.cursor(), tree.root());
        assert_eq!(tree.to_undo(), None);
        assert_eq!(tree.to_redo(), None);
    }

    #[test]
    fn pushing_moves_onto_what_was_pushed() {
        let mut tree = tree();

        let a = tree.push("a");
        let b = tree.push("b");

        assert_eq!(tree.cursor(), b);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.get(b).unwrap().parent, Some(a));
        assert_eq!(tree.value(a), Some(&"a"));
    }

    #[test]
    fn undo_walks_back_and_redo_walks_forward_again() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");

        assert_eq!(tree.to_undo(), Some(b));
        tree.set_cursor(a);
        assert_eq!(tree.to_undo(), Some(a));
        assert_eq!(tree.to_redo(), Some(b));

        tree.set_cursor(tree.root());
        assert_eq!(tree.to_undo(), None);
        assert_eq!(tree.to_redo(), Some(a));
    }

    /// The one the whole shape exists for: going back and then doing something
    /// else keeps both, rather than throwing the first away.
    #[test]
    fn going_back_and_forward_differently_makes_a_branch() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");

        tree.set_cursor(a);
        let c = tree.push("c");

        assert_eq!(tree.len(), 3, "nothing is thrown away");
        assert_eq!(tree.get(a).unwrap().children, vec![b, c]);
        assert_eq!(tree.value(b), Some(&"b"), "the branch left is still whole");
        assert_eq!(tree.cursor(), c);
    }

    /// Redo takes the way just left, not the newest one, because retracing is
    /// what somebody at a fork with the cursor moved back is doing.
    #[test]
    fn redo_takes_the_branch_last_gone_down() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        tree.set_cursor(a);
        let c = tree.push("c");

        // Newest child is c, and the cursor came back along c.
        tree.set_cursor(a);
        assert_eq!(tree.to_redo(), Some(c));

        // Having gone down b, redo from a offers b again.
        tree.set_cursor(b);
        tree.set_cursor(a);
        assert_eq!(tree.to_redo(), Some(b));
    }

    #[test]
    fn a_route_along_one_line_is_all_one_way() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        let c = tree.push("c");

        assert_eq!(
            tree.route(a),
            vec![(c, Way::Back), (b, Way::Back)],
            "back to a runs c and b backwards, and not a itself"
        );

        tree.set_cursor(a);
        assert_eq!(tree.route(c), vec![(b, Way::Forward), (c, Way::Forward)]);
    }

    /// A click in the panel on the other branch: up to where they meet, then
    /// down the other side.
    #[test]
    fn a_route_across_a_fork_goes_up_then_down() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        let c = tree.push("c");
        tree.set_cursor(a);
        let d = tree.push("d");
        let e = tree.push("e");

        // Now at e, on the far branch. Going to c means undoing e and d, then
        // doing b and c.
        assert_eq!(
            tree.route(c),
            vec![
                (e, Way::Back),
                (d, Way::Back),
                (b, Way::Forward),
                (c, Way::Forward)
            ]
        );
    }

    #[test]
    fn a_route_to_where_we_already_are_does_nothing() {
        let mut tree = tree();
        let a = tree.push("a");

        assert!(tree.route(a).is_empty());
        assert!(
            tree.route(404).is_empty(),
            "and neither does a node that went"
        );
    }

    #[test]
    fn depth_counts_the_steps_back_to_the_beginning() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");

        assert_eq!(tree.depth(tree.root()), 0);
        assert_eq!(tree.depth(a), 1);
        assert_eq!(tree.depth(b), 2);
    }

    /// A straight run is not indented at all: it is the depth that grows, and
    /// indenting by that would push a day's work off the side of the panel.
    #[test]
    fn a_straight_run_is_never_indented() {
        let mut tree = tree();
        let mut last = tree.root();

        for _ in 0..50 {
            last = tree.push("x");
        }

        assert_eq!(tree.depth(last), 50);
        assert_eq!(tree.branch_depth(last), 0);
    }

    /// A branch is one level in, and what hangs off it stays at that level.
    #[test]
    fn a_branch_is_one_level_in() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        tree.set_cursor(a);
        let c = tree.push("c");
        let d = tree.push("d");

        assert_eq!(tree.branch_depth(b), 0, "the first way tried is the line");
        assert_eq!(tree.branch_depth(c), 1);
        assert_eq!(tree.branch_depth(d), 1, "and what follows it stays there");
    }

    #[test]
    fn a_node_is_its_own_ancestor_and_the_root_is_everyones() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        tree.set_cursor(a);
        let c = tree.push("c");

        assert!(tree.is_ancestor(a, b));
        assert!(tree.is_ancestor(b, b));
        assert!(tree.is_ancestor(tree.root(), c));
        assert!(
            !tree.is_ancestor(b, c),
            "a branch is not on the other's line"
        );
        assert!(!tree.is_ancestor(c, a));
    }

    #[test]
    fn the_list_is_in_the_order_things_happened() {
        let mut tree = tree();
        let a = tree.push("a");
        tree.set_cursor(tree.root());
        let b = tree.push("b");

        let order: Vec<NodeId> = tree.in_order().map(|(id, _)| id).collect();

        assert_eq!(
            order,
            vec![tree.root(), a, b],
            "creation order, not tree order, because that is when they happened"
        );
    }

    #[test]
    fn trimming_forgets_the_oldest_first() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        let c = tree.push("c");

        tree.trim(2);

        assert_eq!(tree.len(), 2);
        assert!(tree.get(a).is_none(), "the oldest went");
        assert!(tree.get(b).is_some() && tree.get(c).is_some());
        assert_eq!(
            tree.get(b).unwrap().parent,
            Some(tree.root()),
            "and what was after it is joined to what was before it"
        );
    }

    /// A trim must not cost more than the deed it forgets: a branch hanging
    /// off the dropped node is still there afterwards.
    #[test]
    fn trimming_keeps_the_branches_of_what_it_forgets() {
        let mut tree = tree();
        let a = tree.push("a");
        let b = tree.push("b");
        tree.set_cursor(a);
        let c = tree.push("c");

        tree.trim(2);

        assert!(tree.get(a).is_none());
        assert_eq!(tree.get(b).unwrap().parent, Some(tree.root()));
        assert_eq!(tree.get(c).unwrap().parent, Some(tree.root()));
        assert_eq!(tree.get(tree.root()).unwrap().children, vec![b, c]);
    }

    #[test]
    fn trimming_never_drops_where_we_are() {
        let mut tree = tree();
        let a = tree.push("a");
        let _b = tree.push("b");
        tree.set_cursor(a);

        tree.trim(1);

        assert!(tree.get(a).is_some(), "the cursor is still somewhere");
        assert_eq!(tree.cursor(), a);
    }

    #[test]
    fn a_limit_of_nought_keeps_everything() {
        let mut tree = tree();
        for _ in 0..50 {
            tree.push("x");
        }

        tree.trim(0);

        assert_eq!(tree.len(), 50);
    }

    /// An index means one node for the life of the tree, so a panel holding
    /// one across a trim cannot come to be pointing at something else.
    #[test]
    fn an_index_is_never_reused() {
        let mut tree = tree();
        let a = tree.push("a");
        tree.push("b");

        tree.trim(1);
        assert!(tree.get(a).is_none());

        let c = tree.push("c");
        assert_ne!(c, a, "the empty slot was not handed out again");
    }
}

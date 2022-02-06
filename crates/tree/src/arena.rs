pub type NodeHandle = usize;

pub struct Node<Data> {
    pub data: Data,
    parent: Option<NodeHandle>,
    children: Vec<NodeHandle>,
}

impl<Data> Node<Data> {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            parent: None,
            children: Vec::new(),
        }
    }
    pub fn parent(&self) -> &Option<NodeHandle> { &self.parent }
    pub fn children(&self) -> &Vec<NodeHandle> { &self.children }
    pub fn has_parent(&self) -> bool { self.parent.is_some() }
    pub fn has_children(&self) -> bool { !self.children.is_empty() }
    pub fn is_root(&self) -> bool { self.parent.is_none() }
    pub fn is_leaf(&self) -> bool { self.children.is_empty() }
    pub fn is_internal(&self) -> bool { !(self.is_root() || self.is_leaf()) }
}

enum VisitPlan {
    Next,
    Skip,
    Finish,
}

pub struct VisitContext {
    pub depth: usize,
    next_plan: VisitPlan,
}

impl VisitContext {
    fn new(depth: usize) -> Self {
        Self { depth, next_plan: VisitPlan::Next }
    }
    pub fn skip(&mut self) { self.next_plan = VisitPlan::Skip }
    pub fn finish(&mut self) { self.next_plan = VisitPlan::Finish }
}

pub trait NodeArena<Data>: std::ops::Index<NodeHandle> + std::ops::IndexMut<NodeHandle> {
    fn get(&self, handle: NodeHandle) -> Option<&Node<Data>>;
    fn get_mut(&mut self, handle: NodeHandle) -> Option<&mut Node<Data>>;
    fn contains(&self, handle: NodeHandle) -> bool { self.get(handle).is_some() }
    fn create(&mut self, data: Data) -> NodeHandle;
    /// Destroys the node and its children.
    fn destroy(&mut self, handle: NodeHandle);
    /// ## Panics
    /// Panics if the `parent` node doesn't exist.
    fn append_child(&mut self, parent: NodeHandle, data: Data) -> NodeHandle {
        debug_assert!(self.contains(parent));
        let child = self.create(data);
        self.get_mut(parent).unwrap().children.push(child);
        self.get_mut(child).unwrap().parent = Some(parent);
        child
    }
    /// Detaches the node from its parent.
    /// ## Panics
    /// Panics if the specified node doesn't exist.
    fn detach(&mut self, handle: NodeHandle) -> bool {
        if let Some(parent) = self.get(handle).unwrap().parent {
            self.get_mut(handle).unwrap().parent = None;
            let children = &mut self.get_mut(parent).unwrap().children;
            let i = children.iter().position(|h| h.eq(&handle));
            debug_assert!(i.is_some());
            children.remove(i.unwrap());
            true
        } else {
            false
        }
    }
    /// ## Panics
    /// Panics if the visited nodes do not exist.
    fn visit_depth_first(&self, start: NodeHandle, mut visitor: impl FnMut(&Self, NodeHandle, &mut VisitContext)) {
        let mut current = start;
        let mut depth = 0;
        let mut indices = Vec::new();
        loop {
            debug_assert!(self.contains(current));
            let mut ctx = VisitContext::new(depth);
            visitor(self, current, &mut ctx);
            match ctx.next_plan {
                VisitPlan::Next => {
                    if let Some(next) = self.get(current).unwrap().children.get(0) {
                        current = *next;
                        depth += 1;
                        indices.push(0);
                        continue;
                    }
                }
                VisitPlan::Skip => {}
                VisitPlan::Finish => return,
            }
            loop {
                if current == start {
                    return;
                }
                let c = self.get(current).unwrap();
                let p = self.get(c.parent.unwrap()).unwrap();
                indices[depth - 1] += 1;
                if let Some(next) = p.children.get(indices[depth - 1]) {
                    current = *next;
                    break;
                }
                current = c.parent.unwrap();
                depth -= 1;
                indices.pop();
            }
        }
    }
    fn route(&self, node: NodeHandle) -> Vec<NodeHandle> {
        let mut r = Vec::new();
        let mut c = node;
        loop {
            r.push(c);
            if let Some(p) = self.get(c).unwrap().parent {
                c = p;
            } else {
                break;
            }
        }
        r.reverse();
        r
    }
}

macro_rules! impl_arena_index_ops {
    ($arena_type:ident) => {
        impl<Data> std::ops::Index<NodeHandle> for $arena_type<Data> {
            type Output = Node<Data>;
            fn index(&self, index: NodeHandle) -> &Self::Output {
                self.get(index).unwrap()
            }
        }
        impl<Data> std::ops::IndexMut<NodeHandle> for $arena_type<Data> {
            fn index_mut(&mut self, index: NodeHandle) -> &mut Self::Output {
                self.get_mut(index).unwrap()
            }
        }
    }
}

pub struct VecNodeArena<Data> {
    handle_indices: Vec<Option<NodeHandle>>,
    recycled_indices: Vec<usize>,
    nodes: Vec<Node<Data>>,
}

impl<Data> Default for VecNodeArena<Data> {
    fn default() -> Self {
        Self {
            handle_indices: Vec::new(),
            recycled_indices: Vec::new(),
            nodes: Vec::new(),
        }
    }
}

impl<Data> VecNodeArena<Data> {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn used_len(&self) -> usize { self.nodes.len() - self.recycled_indices.len() }
    pub fn recycled_len(&self) -> usize { self.recycled_indices.len() }
    pub fn shrink(&mut self) { todo!() }
}

impl<Data> NodeArena<Data> for VecNodeArena<Data> {
    fn get(&self, handle: NodeHandle) -> Option<&Node<Data>> {
        if let Some(Some(i)) = self.handle_indices.get(handle) {
            Some(&self.nodes[*i])
        } else {
            None
        }
    }
    fn get_mut(&mut self, handle: NodeHandle) -> Option<&mut Node<Data>> {
        if let Some(Some(i)) = self.handle_indices.get(handle) {
            Some(&mut self.nodes[*i])
        } else {
            None
        }
    }
    fn contains(&self, handle: NodeHandle) -> bool {
        matches!(self.handle_indices.get(handle), Some(Some(_)))
    }
    fn create(&mut self, data: Data) -> NodeHandle {
        if let Some(i) = self.recycled_indices.pop() {
            debug_assert!(self.handle_indices[i].is_none());
            self.nodes[i] = Node::new(data);
            self.handle_indices[i] = Some(i);
            return i;
        }
        debug_assert_eq!(self.nodes.len(), self.handle_indices.len());
        self.nodes.push(Node::new(data));
        let handle = self.nodes.len() - 1;
        self.handle_indices.push(Some(handle));
        handle
    }
    fn destroy(&mut self, handle: NodeHandle) {
        let mut handles = Vec::new();
        self.visit_depth_first(handle, |_, h, _| {
            handles.push(h);
        });
        for h in handles.iter() {
            self.handle_indices[*h] = None;
            self.recycled_indices.push(*h);
        }
    }
}

impl_arena_index_ops!(VecNodeArena);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut arena: VecNodeArena<u8> = Default::default();

        let root = arena.create(1);
        assert_eq!(1, arena[root].data);
        assert!(arena[root].is_root());
        assert!(arena[root].is_leaf());
        assert!(!arena[root].is_internal());

        let n1 = arena.append_child(root, 10);
        assert_eq!(1, arena[root].children.len());
        assert_eq!(&[n1], arena[root].children.as_slice());
        assert!(arena[root].is_root());
        assert!(!arena[root].is_leaf());
        assert!(!arena[root].is_internal());
        assert_eq!(10, arena[n1].data);
        assert_eq!(Some(root), arena[n1].parent);
        assert!(!arena[n1].is_root());
        assert!(arena[n1].is_leaf());
        assert!(!arena[n1].is_internal());

        let _n1_1 = arena.append_child(n1, 100);
        assert!(arena[n1].is_internal());

        let n2 = arena.append_child(root, 20);

        assert!(arena.detach(n1));
        assert!(arena[root].is_root());
        assert!(arena[n1].is_root());

        assert_eq!(0, arena.recycled_len());
        arena.destroy(root);
        assert_eq!(2, arena.recycled_len());
        assert!(!arena.contains(root));
        assert!(!arena.contains(n2));
        assert!(arena.contains(n1));

        let _n1_2 = arena.append_child(n1, 200);
        assert_eq!(1, arena.recycled_len());
    }

    #[test]
    fn test_visit() {
        let mut arena: VecNodeArena<u8> = Default::default();
        let root = arena.create(1);
        let n1 = arena.append_child(root, 10);
        let n2 = arena.append_child(root, 20);
        let n1_1 = arena.append_child(n1, 10);
        let n1_2 = arena.append_child(n1, 10);
        let mut handles = Vec::new();
        arena.visit_depth_first(root, |_, handle, _| {
            handles.push(handle);
        });
        assert_eq!(&[root, n1, n1_1, n1_2, n2], handles.as_slice());
    }
}

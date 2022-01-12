use std::error::Error;
use std::fmt::Display;

pub struct Node<NodeHandle, Data> {
    pub parent: Option<NodeHandle>,
    pub children: Vec<NodeHandle>,
    pub data: Data,
}

impl<NodeHandle, Data> Node<NodeHandle, Data> {
    pub fn new(parent: Option<NodeHandle>, data: Data) -> Self {
        Self {
            parent,
            children: Vec::new(),
            data,
        }
    }
    pub fn is_root(&self) -> bool { self.parent.is_none() }
    pub fn is_leaf(&self) -> bool { self.children.is_empty() }
    pub fn is_internal(&self) -> bool { !(self.is_root() || self.is_leaf()) }
}

pub trait NodeArena<Data> {
    type NodeHandle: Sized + Copy + Eq + Display;
    fn get(&self, handle: Self::NodeHandle) -> Option<&Node<Self::NodeHandle, Data>>;
    fn get_mut(&mut self, handle: Self::NodeHandle) -> Option<&mut Node<Self::NodeHandle, Data>>;
    fn contains(&self, handle: Self::NodeHandle) -> bool { self.get(handle).is_some() }
    fn append(&mut self, parent: Option<Self::NodeHandle>, data: Data) -> Result<Self::NodeHandle, Box<dyn Error>>;
    fn visit_depth_first(&self, start: Self::NodeHandle, mut visitor: impl FnMut(&Self, Self::NodeHandle, usize) -> bool) -> Result<(), Box<dyn Error>> {
        let mut current = start;
        let mut depth = 0;
        let mut indices = Vec::new();
        loop {
            if !self.contains(current) {
                return Err(format!("The node ({}) to be visited not found.", current).into());
            }
            if !visitor(self, current, depth) {
                break;
            }
            if let Some(next) = self.get(current).unwrap().children.get(0) {
                current = *next;
                depth += 1;
                indices.push(0);
                continue;
            }
            loop {
                if current == start {
                    return Ok(());
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
        Ok(())
    }
}

macro_rules! impl_arena_index_ops {
    ($handle_type:ty, $arena_type:ident) => {
        impl<Data> std::ops::Index<$handle_type> for $arena_type<Data> {
            type Output = Node<$handle_type, Data>;
            fn index(&self, index: $handle_type) -> &Self::Output {
                self.get(index).unwrap()
            }
        }
        impl<Data> std::ops::IndexMut<$handle_type> for $arena_type<Data> {
            fn index_mut(&mut self, index: $handle_type) -> &mut Self::Output {
                self.get_mut(index).unwrap()
            }
        }
    }
}

#[derive(Default)]
pub struct VecNodeArena<Data> {
    nodes: Vec<Node<usize, Data>>,
}

impl<Data> NodeArena<Data> for VecNodeArena<Data> {
    type NodeHandle = usize;
    fn get(&self, handle: Self::NodeHandle) -> Option<&Node<Self::NodeHandle, Data>> { self.nodes.get(handle) }
    fn get_mut(&mut self, handle: Self::NodeHandle) -> Option<&mut Node<Self::NodeHandle, Data>> { self.nodes.get_mut(handle) }
    fn contains(&self, handle: Self::NodeHandle) -> bool { handle < self.nodes.len() }
    fn append(&mut self, parent: Option<Self::NodeHandle>, data: Data) -> Result<Self::NodeHandle, Box<dyn Error>> {
        if let Some(ph) = parent {
            if matches!(self.get(ph), None) {
                return Err("parent not found".into());
            }
        }
        self.nodes.push(Node::new(parent, data));
        let handle = self.nodes.len() - 1;
        if let Some(ph) = parent {
            let pn = self.get_mut(ph).unwrap();
            pn.children.push(handle);
        }
        Ok(handle)
    }
}

impl_arena_index_ops!(usize, VecNodeArena);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut arena: VecNodeArena<u8> = Default::default();

        let root = arena.append(None, 1).unwrap();
        assert_eq!(1, arena[root].data);
        assert!(arena[root].is_root());
        assert!(arena[root].is_leaf());
        assert!(!arena[root].is_internal());

        let n1 = arena.append(Some(root), 10).unwrap();
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

        arena.append(Some(n1), 100).unwrap();
        assert!(arena[n1].is_internal());
    }

    #[test]
    fn test_visit() {
        let mut arena: VecNodeArena<u8> = Default::default();
        let root = arena.append(None, 1).unwrap();
        let c1 = arena.append(Some(root), 10).unwrap();
        let c2 = arena.append(Some(root), 20).unwrap();
        let c1_1 = arena.append(Some(c1), 10).unwrap();
        let c1_2 = arena.append(Some(c1), 10).unwrap();
        let mut handles = Vec::new();
        arena.visit_depth_first(root, |_, handle, _| {
            handles.push(handle);
            true
        }).unwrap();
        assert_eq!(&[root, c1, c1_1, c1_2, c2], handles.as_slice());
    }
}

use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::cmp::Ordering;

pub struct Node<Data> {
    pub parent: Option<Weak<RefCell<Node<Data>>>>,
    pub children: Vec<Rc<RefCell<Node<Data>>>>,
    pub data: Data,
}

impl<Data> Node<Data> {
    pub fn new(parent: Option<Weak<RefCell<Node<Data>>>>, data: Data) -> Self {
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

pub fn new<Data>(data: Data) -> Rc<RefCell<Node<Data>>> {
    Rc::new(RefCell::new(Node::new(None, data)))
}

pub fn get<'a, It: Iterator<Item=&'a usize>, Data>(node: &Rc<RefCell<Node<Data>>>, indices_it: It) -> Option<Rc<RefCell<Node<Data>>>> {
    let mut node = Rc::clone(node);
    for idx in indices_it {
        let child = node.borrow().children.get(*idx).map(Rc::clone);
        if let Some(child) = child {
            node = child;
        } else {
            return None;
        }
    }
    Some(node)
}

pub fn append_child<Data>(parent: &Rc<RefCell<Node<Data>>>, child_data: Data) -> Rc<RefCell<Node<Data>>> {
    let child = Rc::new(RefCell::new(Node::new(Some(Rc::downgrade(parent)), child_data)));
    parent.borrow_mut().children.push(child.clone());
    child
}

pub struct ChildNodeIterator<'a, Data, It: Iterator<Item=&'a usize>> {
    node: Rc<RefCell<Node<Data>>>,
    path: It,
}

impl<'a, Data, It: Iterator<Item=&'a usize>> ChildNodeIterator<'a, Data, It> {
    pub fn new(node: &Rc<RefCell<Node<Data>>>, path: It) -> Self {
        Self {
            node: Rc::clone(node),
            path,
        }
    }
}

impl<'a, Data, It: Iterator<Item=&'a usize>> Iterator for ChildNodeIterator<'a, Data, It> {
    type Item = Rc<RefCell<Node<Data>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.path.next() {
            let node = self.node.borrow().children.get(*idx).cloned();
            if let Some(node) = node {
                self.node = Rc::clone(&node);
                Some(node)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Path {
    pub indices: Vec<usize>,
}

impl Path {
    pub fn new(indices: Vec<usize>) -> Self {
        Self { indices }
    }
    pub fn len(&self) -> usize {
        self.indices.len()
    }
    pub fn iter(&self) -> std::slice::Iter<usize> {
        self.indices.iter()
    }
    pub fn child_node_iter<Data>(&self, node: &Rc<RefCell<Node<Data>>>) -> ChildNodeIterator<Data, std::slice::Iter<usize>> {
        ChildNodeIterator::new(node, self.iter())
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering { self.indices.len().cmp(&other.indices.len()) }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

#[derive(Debug, Copy, Clone)]
pub enum VisitPlan {
    Children,
    Sibling,
    End,
}

#[derive(Debug, Clone, Default)]
pub struct VisitState {
    pub path: Path,
}

pub type Visitor<Data, Context> = fn(node: &Rc<RefCell<Node<Data>>>, ctx: &mut Context, state: &VisitState) -> VisitPlan;

pub fn visit<Context, Data>(node: &Rc<RefCell<Node<Data>>>, ctx: &mut Context, visitor: Visitor<Data, Context>) {
    fn visit_internal<Data, Context>(node: &Rc<RefCell<Node<Data>>>, ctx: &mut Context, visitor: Visitor<Data, Context>, state: &VisitState) -> VisitPlan {
        if matches!(visitor(node, ctx, state), VisitPlan::Children) {
            let children = node.borrow().children.clone();
            let mut state = state.clone();
            state.path.indices.push(0); // dummy
            for (idx, child) in children.iter().enumerate() {
                *state.path.indices.last_mut().unwrap() = idx;
                if matches!(visit_internal(&child.clone(), ctx, visitor, &state), VisitPlan::End) {
                    return VisitPlan::End;
                }
            }
        }
        VisitPlan::Children
    }
    visit_internal(node, ctx, visitor, &Default::default());
}

pub fn get_all_paths_to_leaves<Data>(node: &Rc<RefCell<Node<Data>>>) -> Vec<Path> {
    let mut paths = vec![];
    visit(node, &mut paths, |node, ctx, state| {
        let node = node.borrow();
        if node.is_leaf() {
            ctx.push(state.path.clone());
        }
        VisitPlan::Children
    });
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let root = new(100);
        let c1 = append_child(&root, 110);
        append_child(&c1, 111);
        append_child(&root, 120);

        struct Context {
            path: Path,
        }
        let mut ctx = Context { path: Default::default() };

        visit(&root, &mut ctx, |_node, ctx, state| {
            if ctx.path.len() < state.path.len() {
                ctx.path = state.path.clone();
            }
            // println!("{}{}", " ".repeat(state.path.len() * 2), node.data);
            VisitPlan::Children
        });

        assert_eq!(vec![0, 0], ctx.path.indices);

        let found = get(&root, ctx.path.iter());
        assert_eq!(Some(111), found.map(|node| node.borrow().data));

        let values = ChildNodeIterator::new(&root, ctx.path.iter())
            .map(|node| node.borrow().data)
            .collect::<Vec<_>>();
        assert_eq!(vec![110, 111], values);

        assert_eq!(vec![
            Path::new(vec![0, 0]),
            Path::new(vec![1]),
        ], get_all_paths_to_leaves(&root));
    }
}

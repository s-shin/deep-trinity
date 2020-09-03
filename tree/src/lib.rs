use std::rc::{Rc, Weak};
use std::cell::{RefCell};

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
}

pub fn new<Data>(data: Data) -> Rc<RefCell<Node<Data>>> {
    Rc::new(RefCell::new(Node::new(None, data)))
}

pub fn get<'a, It: Iterator<Item=&'a usize>, Data>(node: &Rc<RefCell<Node<Data>>>, path_it: It) -> Option<Rc<RefCell<Node<Data>>>> {
    let mut node = Rc::clone(node);
    for idx in path_it {
        let child = node.borrow().children.get(*idx).map(Rc::clone);
        if let Some(child) = child {
            node = child;
        } else {
            return None;
        }
    }
    Some(node)
}


#[derive(Debug, Copy, Clone)]
pub enum VisitPlan {
    Children,
    Sibling,
    End,
}

#[derive(Debug, Clone, Default)]
pub struct VisitState {
    pub path: Vec<usize>,
}

pub type Visitor<Data, Context> = fn(node: &mut Node<Data>, ctx: &mut Context, state: &VisitState) -> VisitPlan;

pub fn append_child<Data>(parent: &Rc<RefCell<Node<Data>>>, child_data: Data) -> Rc<RefCell<Node<Data>>> {
    let child = Rc::new(RefCell::new(Node::new(Some(Rc::downgrade(parent)), child_data)));
    parent.borrow_mut().children.push(child.clone());
    child
}

pub fn visit<Context, Data>(node: &Rc<RefCell<Node<Data>>>, ctx: &mut Context, visitor: Visitor<Data, Context>) {
    fn visit_internal<Data, Context>(node: Rc<RefCell<Node<Data>>>, ctx: &mut Context, visitor: Visitor<Data, Context>, state: &VisitState) -> VisitPlan {
        if matches!(visitor(&mut node.borrow_mut(), ctx, state), VisitPlan::Children) {
            let children = node.borrow().children.clone();
            let mut state = state.clone();
            state.path.push(0); // dummy
            for (idx, child) in children.iter().enumerate() {
                *state.path.last_mut().unwrap() = idx;
                if matches!(visit_internal(child.clone(), ctx, visitor, &state), VisitPlan::End) {
                    return VisitPlan::End;
                }
            }
        }
        VisitPlan::Children
    }
    visit_internal(Rc::clone(node), ctx, visitor, &Default::default());
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
            path: Vec<usize>,
        }
        let mut ctx = Context { path: vec![] };

        visit(&root, &mut ctx, |_node, ctx, state| {
            if ctx.path.len() < state.path.len() {
                ctx.path = state.path.clone();
            }
            // println!("{}{}", " ".repeat(state.path.len() * 2), node.data);
            VisitPlan::Children
        });

        assert_eq!(vec![0, 0], ctx.path);

        let found = get(&root, ctx.path.iter());
        assert_eq!(Some(111), found.map(|node| node.borrow().data));
    }
}

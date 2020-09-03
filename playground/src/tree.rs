// use std::collections::BTreeMap;
//
// pub type NodeId = usize;
//
// pub struct Node<Data> {
//     parent: Option<NodeId>,
//     children: Vec<NodeId>,
//     data: Data,
// }
//
// struct Arena<Data> {
//     last_node_id: NodeId,
//     nodes: BTreeMap<NodeId, Node<Data>>,
// }
//
// impl<Data> Arena<Data> {
//     fn new() -> Self {
//         Self {
//             last_node_id: 0,
//             nodes: BTreeMap::new(),
//         }
//     }
//     fn new_node(&mut self, parent: Option<NodeId>, data: Data) -> NodeId {
//         self.last_node_id += 1;
//         let id = self.last_node_id;
//         let node = Node {
//             parent,
//             children: vec![],
//             data,
//         };
//         self.nodes.insert(id, node);
//         id
//     }
//     fn new_root(&mut self, data: Data) -> NodeId { self.new_node(None, data) }
//     fn new_child(&mut self, parent: NodeId, data: Data) -> NodeId { self.new_node(Some(parent), data) }
//     fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node<Data>> {
//         self.nodes.get_mut(&id)
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test() {
//         let mut arena = Arena::new();
//         let root_node_id = arena.new_node(None, 100);
//         let mut children = vec![
//             arena.new_child(root_node_id, 200),
//             arena.new_child(root_node_id, 300),
//         ];
//         let root = arena.get_node_mut(root_node_id).unwrap();
//         root.children.append(&mut children);
//     }
// }

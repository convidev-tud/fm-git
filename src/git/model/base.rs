pub trait NodeBase {
    fn new<S: Into<String>>(name: S) -> Self;
    fn get_name(&self) -> &String;
    fn insert_path(&mut self, path: Vec<&str>);
}

#[derive(Clone, Debug)]
pub struct Node<ChildType: NodeBase> {
    name: String,
    children: Vec<ChildType>,
}

impl<ChildType: NodeBase> Node<ChildType> {
    pub fn get_child(&self, name: &str) -> Option<&ChildType> {
        self.children.iter().find(|child| child.get_name() == name)
    }
    fn get_child_mut(&mut self, name: &str) -> Option<&mut ChildType> {
        self.children.iter_mut().find(|s| s.get_name() == name)
    }
    fn add_child(&mut self, child: ChildType) {
        self.children.push(child);
    }
}

impl<ChildType: NodeBase> NodeBase for Node<ChildType> {
    fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
        }
    }
    fn get_name(&self) -> &String {
        &self.name
    }

    fn insert_path(&mut self, path: Vec<&str>) {
        if path.is_empty() {
            return;
        }
        let name = path[0];
        let next_child = match self.get_child_mut(name) {
            Some(node) => node,
            None => {
                self.add_child(ChildType::new(name));
                self.get_child_mut(name).unwrap()
            }
        };
        next_child.insert_path(path[1..].to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyNode {
        node: Node<DummyNode>,
    }
    impl NodeBase for DummyNode {
        fn new<S: Into<String>>(name: S) -> Self {
            Self {
                node: Node::new(name),
            }
        }
        fn get_name(&self) -> &String {
            self.node.get_name()
        }
        fn insert_path(&mut self, path: Vec<&str>) {
            self.node.insert_path(path)
        }
    }

    #[test]
    fn test_insert_path() {
        let mut node = Node::<DummyNode>::new("test");
        node.insert_path(vec!["foo", "bar"]);
    }
}

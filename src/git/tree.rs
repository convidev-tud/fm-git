#[derive(Clone, Debug)]
pub struct SymFeatureNode {
    pub name: String,
    children: Vec<SymFeatureNode>,
}
impl SymFeatureNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }
    pub fn add_child(&mut self, name: &str) {
        self.children.push(SymFeatureNode::new(name));
    }
    pub fn iter_children(&self) -> std::slice::Iter<'_, SymFeatureNode> {
        self.children.iter()
    }
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut SymFeatureNode> {
        self.children.iter_mut().find(|s| s.name == name)
    }
    pub fn add_children_recursive(&mut self, qualified_path: Vec<&str>) {
        if qualified_path.is_empty() {
            return;
        }
        let name = qualified_path[0];
        let next_child: &mut SymFeatureNode = match self.get_child_mut(name) {
            Some(node) => node,
            None => {
                self.add_child(name);
                self.get_child_mut(name).unwrap()
            }
        };
        next_child.add_children_recursive(qualified_path[1..].to_vec());
    }
}

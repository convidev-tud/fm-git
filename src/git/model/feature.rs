use crate::git::model::*;

#[derive(Clone, Debug)]
pub struct FeatureRoot {
    features_with_branches: Vec<String>
}
impl FeatureRoot {
    pub fn new() -> Self {
        FeatureRoot { features_with_branches: Vec::new() }
    }
    pub fn iter_features_with_branches(&self) -> impl Iterator<Item=&String> {
        self.features_with_branches.iter()
    }
}
impl NodeTypeBehavior for FeatureRoot {
    fn build_child_from_path(&mut self, path: &Vec<&str>) -> NodeType {
        self.features_with_branches.push(path.join("/"));
        NodeType::Feature(Feature)
    }
}

#[derive(Clone, Debug)]
pub struct Feature;
impl NodeTypeBehavior for Feature {
    fn build_child_from_path(&mut self, _: &Vec<&str>) -> NodeType {
        NodeType::Feature(Self)
    }
}

use crate::git::model::*;

#[derive(Clone, Debug)]
pub struct ProductNode;
impl NodeBase for ProductNode {
    fn new<S: Into<String>>(name: S) -> Self {
        todo!()
    }

    fn get_name(&self) -> &String {
        todo!()
    }
    fn insert_path(&mut self, path: Vec<&str>) {
        todo!()
    }
}

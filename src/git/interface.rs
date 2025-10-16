use crate::git::tree::SymFeatureNode;
use crate::util::u8_to_string;

#[derive(Clone, Debug)]
pub struct GitInterface {
    virtual_root: SymFeatureNode,
}
impl GitInterface {
    pub fn new() -> Self {
        Self {
            virtual_root: SymFeatureNode::new("virtual"),
        }
    }
    pub fn get_complete_tree(&mut self) -> &SymFeatureNode {
        let output = std::process::Command::new("git")
            .arg("branch")
            .output()
            .expect("failed to execute process");
        let all_branches: Vec<String> = u8_to_string(&output.stdout)
            .split("\n")
            .map(|raw_string| raw_string.trim().to_string())
            .collect();
        for branch in all_branches {
            self.virtual_root.add_children_recursive(
                branch
                    .replace("*", "")
                    .replace("_", "")
                    .trim()
                    .split("/")
                    .collect(),
            );
        }
        &self.virtual_root
    }
}

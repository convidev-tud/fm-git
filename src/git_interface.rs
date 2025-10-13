use crate::util::u8_to_string;

#[derive(Clone, Debug)]
pub struct GitInterface {}

impl GitInterface {
    pub fn new() -> Self { Self {} }
    pub fn get_all_branches(&self) -> Vec<String> {
        let output = std::process::Command::new("git")
            .arg("branch")
            .output()
            .expect("failed to execute process");
        u8_to_string(&output.stdout)
            .replace("*", "")
            .split("\n")
            .map(|raw_string| {
                raw_string.trim().to_string()
            })
            .collect()
    }
}
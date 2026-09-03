use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ManifestNode {
    name: String,
    revision: String,
    parent: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    nodes: Vec<ManifestNode>,
}
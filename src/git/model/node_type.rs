use crate::git::model::*;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct WrongNodeTypeError {
    msg: String,
}
impl WrongNodeTypeError {
    pub fn new<S: Into<String>>(msg: S) -> WrongNodeTypeError {
        WrongNodeTypeError { msg: msg.into() }
    }
}
impl Display for WrongNodeTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for WrongNodeTypeError {}

#[derive(Clone, Debug)]
pub enum NodeType {
    Feature,
    Product,
    FeatureRoot,
    ProductRoot,
    Area,
    VirtualRoot,
}

impl NodeType {
    pub fn build_child_from_name(
        &mut self,
        name: &str,
    ) -> Result<NodeType, WrongNodeTypeError> {
        match self {
            Self::Feature => Ok(Self::Feature),
            Self::Product => Ok(Self::Product),
            Self::FeatureRoot => Ok(Self::Feature),
            Self::ProductRoot => Ok(Self::Product),
            Self::VirtualRoot => Ok(Self::Area),
            Self::Area => {
                if name
                    .starts_with(ModelConstants::feature_prefix().as_str())
                {
                    Ok(Self::FeatureRoot)
                } else if name
                    .starts_with(ModelConstants::product_prefix().as_str())
                {
                    Ok(Self::ProductRoot)
                } else {
                    Err(WrongNodeTypeError::new(format!(
                        "'{}' is no valid child of an area node. Valid childs include: feature, product",
                        name
                    )))
                }
            }
        }
    }
}

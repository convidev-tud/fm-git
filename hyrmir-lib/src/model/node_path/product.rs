use std::marker::PhantomData;
use crate::model::node_path::*;
use crate::model::NodeType;

/// Marker for the product root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProductRoot;

impl ValidNodeType for ProductRoot {
    type Classification = Abstract;

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

impl IsUnderArea for ProductRoot {}

/// Marker of a product node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Product<C: NodeClassification> {
    phantom_data: PhantomData<C>,
}

impl<C: NodeClassification> ValidNodeType for Product<C> {
    type Classification = C;

    fn compatible() -> Vec<NodeType> {
        match C::requires_artifact() {
            Some(true) => vec![NodeType::Product(true)],
            Some(false) => vec![NodeType::Product(false)],
            None => vec![NodeType::Product(true), NodeType::Product(false)],
        }
    }
}

impl<C: NodeClassification> IsUnderArea for Product<C> {}

// impl<V: VCS> NodePath<Product<Concrete>, V> {
//     pub fn get_derivation_data(&self) -> Result<DerivationData, dyn Error> {
//         todo!()
//     }
// }
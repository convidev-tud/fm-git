use std::marker::PhantomData;
use crate::model::tree_view::*;
use crate::model::NodeType;

/// Marker for the product root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProductRoot;

impl SymbolicNodeType for ProductRoot {
    type Classification = Abstract;

    fn new() -> Self {
        Self
    }

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

impl<C: NodeClassification> SymbolicNodeType for Product<C> {
    type Classification = C;

    fn new() -> Self {
        Self { phantom_data: PhantomData }
    }

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
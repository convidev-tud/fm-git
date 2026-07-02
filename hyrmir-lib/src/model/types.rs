use crate::model::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Marker for the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;

/// Marker for an area node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

/// Marker for the feature root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FeatureRoot;

/// Marker for the product root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProductRoot;

/// Marker of a feature node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

/// Marker of a product node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Product<C: NodeClassification> {
    phantom_data: PhantomData<C>,
}

/// Placeholder if the exact node type is unknown or does not matter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyType<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

/// Defines a compatible [SymbolicNodeType] as concrete (with associated artifact).
///
/// The trait [IsConcrete] is automatically implemented.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Concrete;

/// Defines a [SymbolicNodeType] as abstract (without associated artifact).
///
/// The trait [IsAbstract] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Abstract;

/// Placeholder if a concretized classification ([Concrete] or [Abstract]) does not matter or is impossible.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyC;

/// Some paths have the option of being concrete (with attached artifacts) or abstract.
/// This is the base trait for this classification.
pub trait NodeClassification: Clone + Debug + Eq + PartialEq + Hash {
    fn requires_artifact() -> Option<bool>;
}

impl NodeClassification for Concrete {
    fn requires_artifact() -> Option<bool> {
        Some(true)
    }
}

impl NodeClassification for Abstract {
    fn requires_artifact() -> Option<bool> {
        Some(false)
    }
}

impl NodeClassification for AnyC {
    fn requires_artifact() -> Option<bool> {
        None
    }
}

/// Symbolic node type base trait.
/// This exists for generic type parameters.
pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {
    type Classification: NodeClassification;
    fn new() -> Self;
    fn compatible() -> Vec<NodeType> {
        vec![]
    }
}

impl SymbolicNodeType for VirtualRoot {
    type Classification = Abstract;

    fn new() -> Self {
        Self
    }

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::VirtualRoot]
    }
}

impl<C: NodeClassification> SymbolicNodeType for Area<C> {
    type Classification = C;

    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

impl SymbolicNodeType for FeatureRoot {
    type Classification = Abstract;

    fn new() -> Self {
        Self
    }

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

impl<C: NodeClassification> SymbolicNodeType for Feature<C> {
    type Classification = C;

    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn compatible() -> Vec<NodeType> {
        match Self::Classification::requires_artifact() {
            Some(true) => vec![NodeType::Feature(true)],
            Some(false) => vec![NodeType::Feature(false)],
            None => vec![NodeType::Feature(true), NodeType::Feature(false)],
        }
    }
}

impl SymbolicNodeType for ProductRoot {
    type Classification = Abstract;

    fn new() -> Self {
        Self
    }

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

/// Defines a type as child of an area.
pub trait UnderArea: SymbolicNodeType {}
impl UnderArea for FeatureRoot {}
impl<C: NodeClassification> UnderArea for Feature<C> {}
impl UnderArea for ProductRoot {}
impl<C: NodeClassification> UnderArea for Product<C> {}

impl<C: NodeClassification> SymbolicNodeType for Product<C> {
    type Classification = C;

    fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }

    fn compatible() -> Vec<NodeType> {
        match C::requires_artifact() {
            Some(true) => vec![NodeType::Product(true)],
            Some(false) => vec![NodeType::Product(false)],
            None => vec![NodeType::Product(true), NodeType::Product(false)],
        }
    }
}

impl<C: NodeClassification> SymbolicNodeType for AnyType<C> {
    type Classification = C;

    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn compatible() -> Vec<NodeType> {
        let mut base = vec![
            NodeType::VirtualRoot,
            NodeType::FeatureRoot,
            NodeType::ProductRoot,
        ];
        match Self::Classification::requires_artifact() {
            Some(true) => {
                base.push(NodeType::Area(true));
                base.push(NodeType::Feature(true));
                base.push(NodeType::Product(true));
            }
            Some(false) => {
                base.push(NodeType::Area(false));
                base.push(NodeType::Feature(false));
                base.push(NodeType::Product(false));
            }
            None => {
                base.push(NodeType::Area(true));
                base.push(NodeType::Feature(true));
                base.push(NodeType::Product(true));
                base.push(NodeType::Area(false));
                base.push(NodeType::Feature(false));
                base.push(NodeType::Product(false));
            }
        }
        base
    }
}

/// Denotes that a [SymbolicNodeType] is concrete (with associated artifact).
///
/// Is automatically implemented if the type uses [Concrete] as parameter.
pub trait IsConcrete: SymbolicNodeType {}
impl<T: SymbolicNodeType<Classification = Concrete>> IsConcrete for T {}

/// Denotes that a [SymbolicNodeType] is abstract (without associated artifact).
///
/// Is automatically implemented if [Abstract] is used as parameter.
pub trait IsAbstract {}
impl<T: SymbolicNodeType<Classification = Abstract>> IsAbstract for T {}






//! Type definitions

use crate::model::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Some paths have the option of being concrete (with attached artifacts) or abstract.
/// This is the base trait for this classification.
pub trait NodeClassification: Clone + Debug + Eq + PartialEq + Hash {
    fn requires_artifact() -> Option<bool>;
}

/// Defines a compatible [SymbolicNodeType] as concrete (with associated artifact).
///
/// The trait [IsConcrete] is automatically implemented.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Concrete;

impl NodeClassification for Concrete {
    fn requires_artifact() -> Option<bool> {
        Some(true)
    }
}

/// Defines a [SymbolicNodeType] as abstract (without associated artifact).
///
/// The trait [IsAbstract] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Abstract;

impl NodeClassification for Abstract {
    fn requires_artifact() -> Option<bool> {
        Some(false)
    }
}

/// Placeholder if a concretized classification ([Concrete] or [Abstract]) does not matter or is impossible.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyC;

impl NodeClassification for AnyC {
    fn requires_artifact() -> Option<bool> {
        None
    }
}

/// Symbolic node type base trait.
/// This exists for generic type parameters.
pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {
    type Classification: NodeClassification;
    fn compatible() -> Vec<NodeType> {
        vec![]
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
pub trait IsAbstract: SymbolicNodeType {}
impl<T: SymbolicNodeType<Classification = Abstract>> IsAbstract for T {}

/// Denotes that an [Abstract] [SymbolicNodeType] may get a branch and become [Concrete] in this process.
pub trait CanBecomeConcrete: IsAbstract {
    type Target: IsConcrete;
}

/// Denotes that a branch may become abstract.
pub trait CanBecomeAbstract: IsConcrete {
    type Target: IsAbstract;
}

/// Defines that a type can create another type.
pub trait CanCreate<T: IsConcrete>: IsConcrete {}

/// Defines a type as child of a channel.
pub trait UnderChannel: SymbolicNodeType {}

/// Marker for the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;

impl SymbolicNodeType for VirtualRoot {
    type Classification = Abstract;

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::ModelRoot]
    }
}

/// Marker for a channel node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Channel<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Channel<C> {
    type Classification = C;

    fn compatible() -> Vec<NodeType> {
        match Self::Classification::requires_artifact() {
            Some(true) => vec![NodeType::ModelRevision(true)],
            Some(false) => vec![NodeType::ModelRevision(false)],
            None => vec![NodeType::ModelRevision(true), NodeType::ModelRevision(false)],
        }
    }
}

impl CanBecomeConcrete for Channel<Abstract> { type Target = Channel<Concrete>; }

/// Marker for the feature root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FeatureRoot;

impl SymbolicNodeType for FeatureRoot {
    type Classification = Abstract;

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::FeatureRoot]
    }
}

impl UnderChannel for FeatureRoot {}

/// Marker of a feature node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Feature<C> {
    type Classification = C;

    fn compatible() -> Vec<NodeType> {
        match Self::Classification::requires_artifact() {
            Some(true) => vec![NodeType::Feature(true)],
            Some(false) => vec![NodeType::Feature(false)],
            None => vec![NodeType::Feature(true), NodeType::Feature(false)],
        }
    }
}

impl<C: NodeClassification> UnderChannel for Feature<C> {}

impl CanBecomeConcrete for Feature<Abstract> { type Target = Feature<Concrete>; }

/// Marker for the product root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProductRoot;

impl SymbolicNodeType for ProductRoot {
    type Classification = Abstract;

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::ProductRoot]
    }
}

impl UnderChannel for ProductRoot {}

/// Marker of a product node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Product<C: NodeClassification> {
    phantom_data: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Product<C> {
    type Classification = C;

    fn compatible() -> Vec<NodeType> {
        match C::requires_artifact() {
            Some(true) => vec![NodeType::Product(true)],
            Some(false) => vec![NodeType::Product(false)],
            None => vec![NodeType::Product(true), NodeType::Product(false)],
        }
    }
}

impl<C: NodeClassification> UnderChannel for Product<C> {}

impl CanBecomeConcrete for Product<Abstract> { type Target = Product<Concrete>; }

/// Placeholder if the exact node type is unknown or does not matter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyType<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for AnyType<C> {
    type Classification = C;

    fn compatible() -> Vec<NodeType> {
        match Self::Classification::requires_artifact() {
            Some(true) => {
                vec![
                    NodeType::ModelRevision(true),
                    NodeType::Feature(true),
                    NodeType::Product(true),
                ]
            }
            Some(false) => {
                vec![
                    NodeType::ModelRevision(false),
                    NodeType::Feature(false),
                    NodeType::Product(false),
                    NodeType::ModelRoot,
                    NodeType::FeatureRoot,
                    NodeType::ProductRoot,
                ]
            }
            None => {
                vec![
                    NodeType::ModelRevision(true),
                    NodeType::Feature(true),
                    NodeType::Product(true),
                    NodeType::ModelRevision(false),
                    NodeType::Feature(false),
                    NodeType::Product(false),
                    NodeType::ModelRoot,
                    NodeType::FeatureRoot,
                    NodeType::ProductRoot,
                ]
            }
        }
    }
}

// Copyright 2026 Hyrmyr Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::model::node::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;


/// Some node types have the option of being concrete (with attached artifacts) or abstract.
/// This is the base trait for this classification.
pub trait NodeClassification: Clone + Debug + Eq + PartialEq + Hash {}

/// Symbolic node type base trait.
/// This exists for generic type parameters.
pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {
    type Classification: NodeClassification;
    fn identifier() -> String;
    fn is_compatible(node: &Node) -> bool {
        Self::is_compatible_to_node(node)
    }
    fn is_compatible_to_node(node: &Node) -> bool;
}

/// Denotes that a [SymbolicNodeType] is concrete (with associated artifact).
///
/// Is automatically implemented if [Concrete] is used as parameter.
pub trait IsConcrete {}

/// Denotes that a [SymbolicNodeType] is abstract (without associated artifact).
///
/// Is automatically implemented if [Abstract] is used as parameter.
pub trait IsAbstract {}

/// Defines a [SymbolicNodeType] as concrete (with associated artifact).
///
/// The trait [IsConcrete] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Concrete;
impl NodeClassification for Concrete {}
impl<T: SymbolicNodeType<Classification=Concrete>> IsConcrete for T {}

/// Defines a [SymbolicNodeType] as abstract (without associated artifact).
///
/// The trait [IsAbstract] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Abstract;
impl NodeClassification for Abstract {}
impl<T: SymbolicNodeType<Classification=Abstract>> IsAbstract for T {}

/// Placeholder if a concretized classification ([Concrete] or [Abstract]) does not matter or is impossible.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyClassification;
impl NodeClassification for AnyClassification {}

/// Defines that the specific node type has children of type [Feature].
pub trait HasFeatureChildren: SymbolicNodeType {}
/// Defines that the specific node type has children of type [Product].
pub trait HasProductChildren: SymbolicNodeType {}
pub trait IsOnOrUnderArea: SymbolicNodeType {}

/// Symbolic representation of a feature node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<T: NodeClassification> {
    _phantom: PhantomData<T>,
}
impl<T: NodeClassification> SymbolicNodeType for Feature<T> {
    type Classification = T;
    fn identifier() -> String {
        format!("{} {}", "", NodeType::Feature(true))
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ConcreteFeature => true,
            _ => false,
        }
    }
}
impl<T: NodeClassification> HasFeatureChildren for Feature<T> {}
impl<T: NodeClassification> IsOnOrUnderArea for Feature<T> {}

/// Symbolic representation of the feature root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FeatureRoot;
impl SymbolicNodeType for FeatureRoot {
    type Classification = Abstract;
    fn identifier() -> String {
        NodeType::FeatureRoot.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::FeatureRoot => true,
            _ => false,
        }
    }
}
impl HasFeatureChildren for FeatureRoot {}
impl IsOnOrUnderArea for FeatureRoot {}

/// Symbolic representation of a product node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Product<T: NodeClassification> {
    _phantom: PhantomData<T>,
}
impl<T: NodeClassification> SymbolicNodeType for Product<T> {
    type Classification = T;
    fn identifier() -> String {
        NodeType::ConcreteProduct.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ConcreteProduct => true,
            _ => false,
        }
    }
}
impl<T: NodeClassification> HasProductChildren for Product<T> {}
impl<T: NodeClassification> IsOnOrUnderArea for Product<T> {}

/// Symbolic representation of the product root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProductRoot;
impl SymbolicNodeType for ProductRoot {
    type Classification = Abstract;
    fn identifier() -> String {
        NodeType::ProductRoot.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ProductRoot => true,
            _ => false,
        }
    }
}
impl HasProductChildren for ProductRoot {}
impl IsOnOrUnderArea for ProductRoot {}

/// Symbolic representation of an area node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<T: NodeClassification> {
    _phantom: PhantomData<T>,
}
impl<T: NodeClassification> SymbolicNodeType for Area<T> {
    type Classification = T;
    fn identifier() -> String {
        NodeType::Area.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Area => true,
            _ => false,
        }
    }
}
impl<T: NodeClassification> IsOnOrUnderArea for Area<T> {}
impl IsGitObject for Area<Concrete> {}

/// Symbolic representation of a temporary node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Temporary<C: NodeClassification> {
    _phantom: PhantomData<C>,
}
impl<C: NodeClassification> SymbolicNodeType for Temporary<C> {
    type Classification = C;
    fn identifier() -> String {
        NodeType::Temporary.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::Temporary => true,
            _ => false,
        }
    }
}
impl<C: NodeClassification> IsGitObject for Temporary<C> {}
impl<C: NodeClassification> IsOnOrUnderArea for Temporary<C> {}

/// Symbolic representation of the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;
impl SymbolicNodeType for VirtualRoot {
    type Classification = Abstract;
    fn identifier() -> String {
        NodeType::VirtualRoot.get_type_name()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::VirtualRoot => true,
            _ => false,
        }
    }
}

/// Placeholder if the exact node type is unknown or does not matter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyNode;
impl SymbolicNodeType for AnyNode {
    type Classification = AnyClassification;
    fn identifier() -> String {
        "any".to_string()
    }

    fn is_compatible_to_node(_node_type: &NodeType) -> bool {
        true
    }
}

/// Placeholder for a concrete node type if the exact type is unknown or does not matter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyConcrete;
impl SymbolicNodeType for AnyConcrete {
    type Classification = Concrete;
    fn identifier() -> String {
        "git object".to_string()
    }

    fn is_compatible_to_node(node_type: &NodeType) -> bool {
        match node_type {
            NodeType::ConcreteFeature
            | NodeType::ConcreteProduct
            | NodeType::Area
            | NodeType::Temporary => true,
            _ => false,
        }
    }
}

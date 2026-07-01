use crate::model::view::*;
use std::marker::PhantomData;





impl IsOrUnderArea for FeatureRoot {}





impl<C: NodeClassification> IsOrUnderArea for Feature<C> {}

pub trait CanMergeWithFeature: SymbolicNodeType {}

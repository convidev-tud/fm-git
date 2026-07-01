use crate::model::NodeType;
use crate::model::view::*;
use std::marker::PhantomData;





impl<C: NodeClassification> IsOrUnderArea for Product<C> {}

// impl<V: VCS> NodePath<Product<Concrete>, V> {
//     pub fn get_derivation_data(&self) -> Result<DerivationData, dyn Error> {
//         todo!()
//     }
// }

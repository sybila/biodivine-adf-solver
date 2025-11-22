use crate::adf_bdds::DirectEncoding;
use ruddy::split::Bdd;
use std::sync::Arc;

#[derive(Clone)]
pub struct ModelSetTwoValued {
    symbolic_set: Bdd,
    encoding: Arc<DirectEncoding>,
}

impl PartialEq for ModelSetTwoValued {
    fn eq(&self, other: &Self) -> bool {
        self.symbolic_set.structural_eq(&other.symbolic_set)
            && Arc::ptr_eq(&self.encoding, &other.encoding)
    }
}

impl Eq for ModelSetTwoValued {}

impl ModelSetTwoValued {
    /// Make a [`ModelSetTwoValued`] from the underlying parts.
    ///
    /// # Panics
    ///
    /// Fails if the `symbolic_set` uses BDD variables that are not used by the given `encoding`.
    pub fn new(symbolic_set: Bdd, encoding: Arc<DirectEncoding>) -> Self {
        assert!(encoding.is_direct_encoded(&symbolic_set));
        ModelSetTwoValued {
            symbolic_set,
            encoding,
        }
    }

    /// Get a reference to the underlying [`Bdd`].
    pub fn symbolic_set(&self) -> &Bdd {
        &self.symbolic_set
    }

    /// Get a reference to the underlying [`DirectEncoding`].
    pub fn encoding(&self) -> &DirectEncoding {
        &self.encoding
    }

    /// Count the models in this set (possibly overflowing to [`f64::INFINITY`]).
    pub fn model_count(&self) -> f64 {
        self.encoding.count_direct_valuations(&self.symbolic_set)
    }
}

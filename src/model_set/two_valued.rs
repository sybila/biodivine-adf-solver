use crate::AdfBdds;
use crate::adf_bdds::DirectEncoding;
use crate::model_set::ModelSet;
use log::trace;
use ruddy::VariableId;
use ruddy::split::Bdd;
use std::collections::BTreeMap;
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

impl ModelSet for ModelSetTwoValued {
    fn symbolic_set(&self) -> &Bdd {
        ModelSetTwoValued::symbolic_set(self)
    }

    fn model_count(&self) -> f64 {
        ModelSetTwoValued::model_count(self)
    }
}

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

    /// Extract the model with the highest number of zeros (the least number of ones).
    ///
    /// # Panics
    ///
    /// The set must not be empty.
    pub fn most_zero_model(&self) -> BTreeMap<VariableId, bool> {
        self.encoding.most_zero_model(&self.symbolic_set)
    }

    /// Returns `true` if this set of models is empty.
    pub fn is_empty(&self) -> bool {
        self.symbolic_set.is_false()
    }

    /// Compute the intersection of two sets.
    pub fn intersect(&self, other: &ModelSetTwoValued) -> ModelSetTwoValued {
        assert!(Arc::ptr_eq(&self.encoding, &other.encoding));

        ModelSetTwoValued {
            symbolic_set: self.symbolic_set.and(&other.symbolic_set),
            encoding: self.encoding.clone(),
        }
    }

    /// Compute the union of two sets.
    pub fn union(&self, other: &ModelSetTwoValued) -> ModelSetTwoValued {
        assert!(Arc::ptr_eq(&self.encoding, &other.encoding));

        ModelSetTwoValued {
            symbolic_set: self.symbolic_set.or(&other.symbolic_set),
            encoding: self.encoding.clone(),
        }
    }

    /// Compute the difference of two sets.
    pub fn minus(&self, other: &ModelSetTwoValued) -> ModelSetTwoValued {
        assert!(Arc::ptr_eq(&self.encoding, &other.encoding));

        ModelSetTwoValued {
            symbolic_set: self.symbolic_set.and(&other.symbolic_set.not()),
            encoding: self.encoding.clone(),
        }
    }

    /// Compute the set of ADF interpretations that have *exactly* `k` statements set to one.
    ///
    /// Under normal circumstances, this should be a relatively fast operation, where the
    /// resulting BDD is linear in the number of statements. Hence, it is currently
    /// not cancellable.
    pub fn mk_exactly_k_one_statements(one_count: usize, encoding: &AdfBdds) -> ModelSetTwoValued {
        let direct = encoding.direct_encoding();
        let direct_vars = direct.var_map().variable_ids().copied().collect::<Vec<_>>();
        let at_most_k_one = Bdd::new_sat_exactly_k(one_count, &direct_vars);
        encoding.mk_two_valued_set(at_most_k_one)
    }

    /// Extend this set with every interpretation that has additional statements fixed to one.
    pub fn extend_with_more_ones(&self) -> ModelSetTwoValued {
        let mut result = self.symbolic_set.clone();
        for (i, var) in self
            .encoding
            .var_map()
            .variable_ids()
            .copied()
            .rev()
            .enumerate()
        {
            let lit = Bdd::new_literal(var, true);
            let nlit = Bdd::new_literal(var, false);

            // Select every model in which we have `var=false`, but there is
            // no equivalent model with `var=true`. Flips `var` on output,
            // meaning we actually get the set of models where `true` is added.

            let adds_true = result
                // Select every space where p_var=false and eliminate p_var
                .binary_op_with_exists(&nlit, ruddy::boolean_operators::And, &[var])
                // Reintroduce it with p_var=true
                .and(&lit);

            if !adds_true.is_false() {
                result = result.or(&adds_true);
                trace!(
                    "Computing models with more ones [{}/{}]: BDD size {}.",
                    i + 1,
                    self.encoding.var_map().size(),
                    result.node_count(),
                );
            }
        }

        ModelSetTwoValued {
            symbolic_set: result,
            encoding: self.encoding.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AdfBdds, Statement};

    fn create_test_adf_bdds() -> AdfBdds {
        let adf_str = r#"
            s(0).
            s(1).
            ac(0, 1).
            ac(1, c(v)).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        AdfBdds::from(&expr_adf)
    }

    // Note: Testing with invalid BDD is difficult because we can't easily access the Arc<DirectEncoding>
    // The new() method validates that the BDD uses only variables from the encoding, so the
    // validation is tested implicitly through successful creation tests.

    #[test]
    fn test_symbolic_set_accessor() {
        let adf = create_test_adf_bdds();
        let true_bdd = ruddy::split::Bdd::new_true();
        let model_set = adf.mk_two_valued_set(true_bdd.clone());

        let retrieved_bdd = model_set.symbolic_set();
        assert!(retrieved_bdd.structural_eq(&true_bdd));
    }

    #[test]
    fn test_encoding_accessor() {
        let adf = create_test_adf_bdds();
        let bdd = ruddy::split::Bdd::new_true();
        let model_set1 = adf.mk_two_valued_set(bdd.clone());
        let model_set2 = adf.mk_two_valued_set(bdd);

        let encoding1 = model_set1.encoding();
        let encoding2 = model_set2.encoding();
        assert_eq!(encoding1.var_map(), encoding2.var_map());
    }

    #[test]
    fn test_model_count_true() {
        let adf = create_test_adf_bdds();
        let true_bdd = ruddy::split::Bdd::new_true();
        let model_set = adf.mk_two_valued_set(true_bdd);

        // True BDD accepts all 2^2 = 4 valuations for 2 statements
        assert_eq!(model_set.model_count(), 4.0);
    }

    #[test]
    fn test_model_count_false() {
        let adf = create_test_adf_bdds();
        let false_bdd = ruddy::split::Bdd::new_false();
        let model_set = adf.mk_two_valued_set(false_bdd);

        // False BDD accepts no valuations
        assert_eq!(model_set.model_count(), 0.0);
    }

    #[test]
    fn test_model_count_single_literal() {
        let adf = create_test_adf_bdds();
        let var_map = adf.direct_encoding().var_map();
        let s0_true = var_map.make_literal(&Statement::from(0), true);
        let model_set = adf.mk_two_valued_set(s0_true);

        // s(0) = true accepts 2 valuations: (T,T) and (T,F)
        assert_eq!(model_set.model_count(), 2.0);
    }

    #[test]
    fn test_model_count_and() {
        let adf = create_test_adf_bdds();
        let var_map = adf.direct_encoding().var_map();
        let s0 = var_map.make_literal(&Statement::from(0), true);
        let s1 = var_map.make_literal(&Statement::from(1), true);
        let and_bdd = s0.and(&s1);
        let model_set = adf.mk_two_valued_set(and_bdd);

        // s(0) AND s(1) accepts only 1 valuation: (T,T)
        assert_eq!(model_set.model_count(), 1.0);
    }

    #[test]
    fn test_clone() {
        let adf = create_test_adf_bdds();
        let bdd = ruddy::split::Bdd::new_true();
        let model_set1 = adf.mk_two_valued_set(bdd);
        let model_set2 = model_set1.clone();

        // Both should have the same model count
        assert_eq!(model_set1.model_count(), model_set2.model_count());
        assert_eq!(model_set1.model_count(), 4.0);

        // Both should be equal
        assert!(model_set1 == model_set2);
    }

    #[test]
    fn test_partial_eq_equal() {
        let adf = create_test_adf_bdds();
        let bdd = ruddy::split::Bdd::new_true();
        let model_set1 = adf.mk_two_valued_set(bdd.clone());
        let model_set2 = adf.mk_two_valued_set(bdd);

        // Both use the same BDD and encoding, so they should be equal
        assert!(model_set1 == model_set2);
    }

    #[test]
    fn test_partial_eq_different_bdd() {
        let adf = create_test_adf_bdds();
        let true_bdd = ruddy::split::Bdd::new_true();
        let false_bdd = ruddy::split::Bdd::new_false();
        let model_set1 = adf.mk_two_valued_set(true_bdd);
        let model_set2 = adf.mk_two_valued_set(false_bdd);

        // Different BDDs should not be equal
        assert!(model_set1 != model_set2);
    }

    #[test]
    fn test_model_count_complex_expression() {
        let adf = create_test_adf_bdds();
        let var_map = adf.direct_encoding().var_map();
        let s0 = var_map.make_literal(&Statement::from(0), true);
        let s1 = var_map.make_literal(&Statement::from(1), true);

        // Create (s(0) AND s(1)) OR (s(0) AND !s(1))
        let s1_neg = s1.not();
        let and1 = s0.and(&s1);
        let and2 = s0.and(&s1_neg);
        let or_bdd = and1.or(&and2);
        let model_set = adf.mk_two_valued_set(or_bdd);

        // This accepts 2 valuations: (T,T) and (T,F)
        assert_eq!(model_set.model_count(), 2.0);
    }
}

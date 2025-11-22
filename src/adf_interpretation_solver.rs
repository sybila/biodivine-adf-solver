use crate::bdd_solver::{BddSolver, DynamicBddSolver};
use crate::{AdfBdds, ModelSetThreeValued, ModelSetTwoValued};
use cancel_this::{Cancellable, is_cancelled};
use log::{debug, info};

pub struct AdfInterpretationSolver {
    solver: DynamicBddSolver,
}

impl<S: BddSolver + 'static> From<S> for AdfInterpretationSolver {
    fn from(value: S) -> Self {
        AdfInterpretationSolver::new(Box::new(value))
    }
}

impl AdfInterpretationSolver {
    /// Create a new `AdfInterpretationSolver` with the given BDD solver.
    pub fn new(solver: DynamicBddSolver) -> Self {
        AdfInterpretationSolver { solver }
    }

    /// Computes the [`ModelSetTwoValued`] of all complete two valued interpretations of this ADF.
    pub fn solve_complete_two_valued(&self, adf: &AdfBdds) -> Cancellable<ModelSetTwoValued> {
        info!("Starting computation of complete two-valued interpretations");

        let direct = adf.direct_encoding();
        let var_map = direct.var_map();

        let mut fixed_point_constraints = Vec::new();
        let total_statements = var_map.statements().count();

        for statement in var_map.statements() {
            is_cancelled!()?;

            // Get the BDD literal for this statement
            let statement_lit = var_map.make_literal(statement, true);

            // If condition does not exist, this is a free statement.
            let Some(condition) = direct.get_condition(statement) else {
                continue;
            };

            // Fixed point constraint: statement <=> condition
            let constraint = statement_lit.iff(condition);

            debug!(
                "Generated constraint of size {} for statement `{}`",
                constraint.node_count(),
                statement
            );

            fixed_point_constraints.push(constraint);
        }

        info!(
            "Generated {} fixed-point constraints from {} statements",
            fixed_point_constraints.len(),
            total_statements
        );

        let result_bdd = self.solver.solve_conjunction(&fixed_point_constraints)?;

        let model_set = adf.mk_two_valued_set(result_bdd);

        info!(
            "Computation complete: found {} complete two-valued interpretations",
            model_set.model_count()
        );

        Ok(model_set)
    }

    pub fn solve_admissible(&self, adf: &AdfBdds) -> Cancellable<ModelSetThreeValued> {
        info!("Starting computation of admissible three-valued interpretations");

        let dual = adf.dual_encoding();
        let var_map = dual.var_map();

        let mut trap_constraints = vec![dual.valid().clone()];
        let total_statements = var_map.statements().count();

        for statement in var_map.statements() {
            is_cancelled!()?;

            // Get the BDD literal for this statement
            let statement_p_lit = var_map.make_positive_literal(statement, true);
            let statement_n_lit = var_map.make_negative_literal(statement, true);

            // If condition does not exist, this is a free statement.
            let Some((p_condition, n_condition)) = dual.get_condition(statement) else {
                continue;
            };

            // If the condition can evaluate to true, the corresponding literal must be also set.
            let p_constraint = p_condition.implies(&statement_p_lit);
            let n_constraint = n_condition.implies(&statement_n_lit);

            debug!(
                "Generated constraints of size {}/{} for statement `{}`",
                p_constraint.node_count(),
                n_constraint.node_count(),
                statement
            );

            trap_constraints.push(p_constraint);
            trap_constraints.push(n_constraint);
        }

        info!(
            "Generated {} trap constraints from {} statements",
            trap_constraints.len(),
            total_statements
        );

        let result_bdd = self.solver.solve_conjunction(&trap_constraints)?;

        let model_set = adf.mk_three_valued_set(result_bdd);

        info!(
            "Computation complete: found {} admissible three-valued interpretations",
            model_set.model_count()
        );

        Ok(model_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bdd_solver::NaiveGreedySolver;

    fn create_test_solver() -> AdfInterpretationSolver {
        AdfInterpretationSolver::from(NaiveGreedySolver::default())
    }

    #[test]
    fn test_solve_simple_adf_constant_true() {
        let solver = create_test_solver();
        let adf_str = r#"
            s(0).
            ac(0, c(v)).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        let adf = AdfBdds::from(&expr_adf);

        let model_set = solver
            .solve_complete_two_valued(&adf)
            .expect("Solving should not be cancelled");

        // For statement 0 with constant true condition:
        // Fixed point: 0 <=> true, which means 0 must be true
        // So there is exactly 1 interpretation: {0: true}
        assert_eq!(model_set.model_count(), 1.0);
    }

    #[test]
    fn test_solve_two_statements() {
        let solver = create_test_solver();
        let adf_str = r#"
            s(0).
            s(1).
            ac(0, 1).
            ac(1, 0).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        let adf = AdfBdds::from(&expr_adf);

        let model_set = solver
            .solve_complete_two_valued(&adf)
            .expect("Solving should not be cancelled");

        // For statements with mutual dependencies:
        // Fixed points: 0 <=> 1 and 1 <=> 0
        // This means: 0 <=> 1, so both must be true or both must be false
        // There are 2 interpretations: {0: true, 1: true} and {0: false, 1: false}
        assert_eq!(model_set.model_count(), 2.0);
    }

    #[test]
    fn test_solve_with_free_statement() {
        let solver = create_test_solver();
        let adf_str = r#"
            s(0).
            s(1).
            ac(0, c(v)).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        let adf = AdfBdds::from(&expr_adf);

        let model_set = solver
            .solve_complete_two_valued(&adf)
            .expect("Solving should not be cancelled");

        // Statement 0 has condition true, so 0 must be true (fixed point: 0 <=> true)
        // Statement 1 has no condition (free), so it can be true or false
        // There are 2 interpretations: {0: true, 1: true} and {0: true, 1: false}
        assert_eq!(model_set.model_count(), 2.0);
    }

    #[test]
    fn test_solve_admissible_simple_constant_true() {
        let solver = create_test_solver();
        let adf_str = r#"
            s(0).
            ac(0, c(v)).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        let adf = AdfBdds::from(&expr_adf);

        let model_set = solver
            .solve_admissible(&adf)
            .expect("Solving should not be cancelled");

        // For statement 0 with constant true condition:
        // Trap constraint: if condition can be true (always), then positive literal must be set
        // This means statement 0 can be true or undefined (both), but not false
        // For one statement: {T}, {U}, or {T,U} are valid three-valued interpretations
        // With the constraint, we must allow T, so we have: {T} and {T,U}
        assert_eq!(model_set.model_count(), 2.0);
    }

    #[test]
    fn test_solve_admissible_two_statements() {
        let solver = create_test_solver();
        let adf_str = r#"
            s(0).
            s(1).
            ac(0, 1).
            ac(1, c(v)).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        let adf = AdfBdds::from(&expr_adf);

        let model_set = solver
            .solve_admissible(&adf)
            .expect("Solving should not be cancelled");

        // Statement 0 depends on 1, statement 1 has constant true
        // This means both statements can be * or 1, but not 0. Valid statements are: **, *1, 11.
        assert_eq!(model_set.model_count(), 3.0);
    }

    #[test]
    fn test_solve_admissible_with_free_statement() {
        let solver = create_test_solver();
        let adf_str = r#"
            s(0).
            s(1).
            ac(0, c(v)).
        "#;
        let expr_adf = crate::AdfExpressions::parse(adf_str).expect("Failed to parse ADF");
        let adf = AdfBdds::from(&expr_adf);

        let model_set = solver
            .solve_admissible(&adf)
            .expect("Solving should not be cancelled");

        // Statement 0 has constant true condition, statement 1 is free
        // Free statements don't add constraints, so we just have the constraint from statement 0
        // Plus the valid constraint requiring at least one dual variable per statement
        assert_eq!(model_set.model_count(), 6.0);
    }
}

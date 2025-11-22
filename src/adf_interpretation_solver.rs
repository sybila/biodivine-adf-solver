use crate::bdd_solver::{BddSolver, DynamicBddSolver};
use crate::{AdfBdds, ModelSetTwoValued};
use cancel_this::{Cancellable, is_cancelled};

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
        let direct = adf.direct_encoding();
        let var_map = adf.direct_encoding().var_map();

        let mut fixed_point_constraints = Vec::new();
        for statement in var_map.statements() {
            is_cancelled!()?;

            // Get the BDD literal for this statement
            let statement_lit = var_map.make_literal(statement, true);

            // Get the condition for this statement (if it exists)
            let constraint = if let Some(condition_bdd) = direct.get_condition(statement) {
                // Fixed point constraint: statement <=> condition
                statement_lit.iff(condition_bdd)
            } else {
                // No condition means the statement is free - it can be true or false.
                // So the fixed point constraint is just "true" (no constraint).
                // We'll skip adding this to avoid unnecessary work.
                continue;
            };

            fixed_point_constraints.push(constraint);
        }

        let result_bdd = self.solver.solve_conjunction(&fixed_point_constraints)?;

        Ok(adf.mk_two_valued_set(result_bdd))
    }
}

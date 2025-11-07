use crate::{ConditionExpression, ExpressionAdf, Statement};
use ruddy::VariableId;
use ruddy::split::Bdd;
use std::collections::BTreeMap;
use std::ops::Index;

/// Maps every [`Statement`] to a single BDD [`VariableId`].
///
/// It is assumed that the BDD variables follow the natural ordering of the statements, but do not
/// necessarily need to use the exact same identifiers.
pub struct DirectMap {
    mapping: BTreeMap<Statement, VariableId>,
}

impl DirectMap {
    /// Create a new DirectMap from an ordered list of statements.
    pub fn new(statements: &[Statement]) -> Self {
        let mapping = statements
            .iter()
            .map(|stmt| {
                let index = u32::try_from(stmt.into_index()).expect("Statement index out of range");
                (*stmt, VariableId::new(index << 2))
            })
            .collect();
        DirectMap { mapping }
    }

    /// Get the BDD variable ID for a statement.
    pub fn get(&self, statement: &Statement) -> Option<VariableId> {
        self.mapping.get(statement).copied()
    }

    /// Get ordered list of all [`Statement`] objects in the map.
    pub fn statements(&self) -> Vec<Statement> {
        self.mapping.keys().copied().collect()
    }
}

impl Index<&Statement> for DirectMap {
    type Output = VariableId;

    fn index(&self, statement: &Statement) -> &Self::Output {
        self.mapping
            .get(statement)
            .expect("Statement not found in DirectMap")
    }
}

impl Index<Statement> for DirectMap {
    type Output = VariableId;

    fn index(&self, statement: Statement) -> &Self::Output {
        &self[&statement]
    }
}

/// Maps every [`Statement`] to two BDD [`VariableId`] objects, one for "positive" and one for
/// "negative" value of [`Statement`].
///
/// It is assumed that the BDD variables follow the natural ordering of the statements
/// (and positive < negative), but do not necessarily need to use the exact same identifiers.
pub struct DualMap {
    mapping: BTreeMap<Statement, (VariableId, VariableId)>,
}

impl DualMap {
    /// Create a new DualMap from an ordered list of statements.
    /// For each statement, two consecutive variable IDs are allocated (positive, then negative).
    pub fn new(statements: &[Statement]) -> Self {
        let mapping = statements
            .iter()
            .map(|stmt| {
                let index = u32::try_from(stmt.into_index()).expect("Statement index out of range");
                let t_var = VariableId::new((index << 2) + 1);
                let f_var = VariableId::new((index << 2) + 2);
                (*stmt, (t_var, f_var))
            })
            .collect();
        DualMap { mapping }
    }

    /// Get the BDD variable IDs (positive, negative) for a statement.
    pub fn get(&self, statement: &Statement) -> Option<(VariableId, VariableId)> {
        self.mapping.get(statement).copied()
    }

    /// Get ordered list of all [`Statement`] objects in the map.
    pub fn statements(&self) -> Vec<Statement> {
        self.mapping.keys().copied().collect()
    }
}

impl Index<&Statement> for DualMap {
    type Output = (VariableId, VariableId);

    fn index(&self, statement: &Statement) -> &Self::Output {
        self.mapping
            .get(statement)
            .expect("Statement not found in DualMap")
    }
}

impl Index<Statement> for DualMap {
    type Output = (VariableId, VariableId);

    fn index(&self, statement: Statement) -> &Self::Output {
        &self[&statement]
    }
}

/// Uses [`DirectMap`] to encode every condition of an ADF directly into a BDD.
///
/// Note that statements can exist in `var_map` that do not have corresponding conditions.
/// These are considered to be "free" statements.
pub struct DirectEncoding {
    var_map: DirectMap,
    conditions: BTreeMap<Statement, Bdd>,
}

impl DirectEncoding {
    /// Get the variable map.
    pub fn var_map(&self) -> &DirectMap {
        &self.var_map
    }

    /// Get the BDD condition for a statement, if it exists.
    pub fn get_condition(&self, statement: &Statement) -> Option<&Bdd> {
        self.conditions.get(statement)
    }

    /// Get all statements that have conditions.
    pub fn conditional_statements(&self) -> impl Iterator<Item = &Statement> {
        self.conditions.keys()
    }
}

/// Uses [`DualMap`] to encode every condition of an ADF into two BDDs, one describing
/// all dual interpretations where the statement can become true, and one describing all dual
/// interpretations where the statement can become false.
///
/// Note that statements can exist in `var_map` that do not have corresponding conditions.
/// These are considered to be "free" statements.
///
/// Also note that for many applications, it is sufficient to encode "free" statements using
/// direct encoding. However, strictly speaking, this is not
pub struct DualEncoding {
    var_map: DualMap,
    conditions: BTreeMap<Statement, (Bdd, Bdd)>,
}

impl DualEncoding {
    /// Get the variable map.
    pub fn var_map(&self) -> &DualMap {
        &self.var_map
    }

    /// Get the BDD conditions (can_be_true, can_be_false) for a statement, if they exist.
    pub fn get_condition(&self, statement: &Statement) -> Option<(&Bdd, &Bdd)> {
        self.conditions.get(statement).map(|(t, f)| (t, f))
    }

    /// Get all statements that have conditions.
    pub fn conditional_statements(&self) -> impl Iterator<Item = &Statement> {
        self.conditions.keys()
    }
}

/// A [`SymbolicAdf`] encodes an ADF symbolically using BDDs.
///
/// Internally, it uses two encodings, depending on use case. Direct encoding uses one BDD
/// variable per statement, while dual encoding uses two.
pub struct SymbolicAdf {
    direct_encoding: DirectEncoding,
    dual_encoding: DualEncoding,
}

impl SymbolicAdf {
    /// Get the direct encoding of this ADF.
    pub fn direct_encoding(&self) -> &DirectEncoding {
        &self.direct_encoding
    }

    /// Get the dual encoding of this ADF.
    pub fn dual_encoding(&self) -> &DualEncoding {
        &self.dual_encoding
    }
}

impl From<&ExpressionAdf> for SymbolicAdf {
    fn from(adf: &ExpressionAdf) -> Self {
        // Get all statements in sorted order
        let statements: Vec<Statement> = adf.statements().copied().collect();

        // Create variable maps
        let direct_map = DirectMap::new(&statements);
        let dual_map = DualMap::new(&statements);

        // Build direct encoding conditions
        let mut direct_conditions = BTreeMap::new();
        for statement in &statements {
            if let Some(expr) = adf.get_condition(statement) {
                let bdd = expression_to_bdd(expr, &direct_map);
                direct_conditions.insert(*statement, bdd);
            }
        }

        // Build dual encoding conditions from direct encoding
        let mut dual_conditions = BTreeMap::new();
        for statement in &statements {
            if let Some(direct_bdd) = direct_conditions.get(statement) {
                let can_be_true = direct_to_dual_encoding(direct_bdd, &direct_map, &dual_map);
                let can_be_false =
                    direct_to_dual_encoding(&direct_bdd.not(), &direct_map, &dual_map);
                dual_conditions.insert(*statement, (can_be_true, can_be_false));
            }
        }

        SymbolicAdf {
            direct_encoding: DirectEncoding {
                var_map: direct_map,
                conditions: direct_conditions,
            },
            dual_encoding: DualEncoding {
                var_map: dual_map,
                conditions: dual_conditions,
            },
        }
    }
}

impl From<ExpressionAdf> for SymbolicAdf {
    fn from(adf: ExpressionAdf) -> Self {
        SymbolicAdf::from(&adf)
    }
}

/// Convert a ConditionExpression to a BDD using direct encoding.
fn expression_to_bdd(expr: &ConditionExpression, var_map: &DirectMap) -> Bdd {
    // Check for constant
    if let Some(value) = expr.as_constant() {
        return if value {
            Bdd::new_true()
        } else {
            Bdd::new_false()
        };
    }

    // Check for statement reference
    if let Some(stmt) = expr.as_statement() {
        let var = var_map[&stmt];
        return Bdd::new_literal(var, true);
    }

    // Check for negation
    if let Some(operand) = expr.as_negation() {
        return expression_to_bdd(operand, var_map).not();
    }

    // Check for AND
    if let Some(operands) = expr.as_and() {
        return operands.iter().fold(Bdd::new_true(), |acc, op| {
            acc.and(&expression_to_bdd(op, var_map))
        });
    }

    // Check for OR
    if let Some(operands) = expr.as_or() {
        return operands.iter().fold(Bdd::new_false(), |acc, op| {
            acc.or(&expression_to_bdd(op, var_map))
        });
    }

    // Check for implication
    if let Some((left, right)) = expr.as_implication() {
        let left_bdd = expression_to_bdd(left, var_map);
        let right_bdd = expression_to_bdd(right, var_map);
        return left_bdd.not().or(&right_bdd);
    }

    // Check for equivalence
    if let Some((left, right)) = expr.as_equivalence() {
        let left_bdd = expression_to_bdd(left, var_map);
        let right_bdd = expression_to_bdd(right, var_map);
        return left_bdd.iff(&right_bdd);
    }

    // Check for exclusive OR
    if let Some((left, right)) = expr.as_exclusive_or() {
        let left_bdd = expression_to_bdd(left, var_map);
        let right_bdd = expression_to_bdd(right, var_map);
        return left_bdd.xor(&right_bdd);
    }

    panic!("Unknown expression type");
}

/// Convert a direct encoding BDD to a dual encoding BDD.
///
/// This applies the principle: for each state variable, we add constraints
/// (state_var => t_var) & (!state_var => f_var) and then existentially quantify state_var.
fn direct_to_dual_encoding(function: &Bdd, direct_map: &DirectMap, dual_map: &DualMap) -> Bdd {
    // Get all statements in the direct map (in reverse order for efficiency)
    let statements = direct_map.statements();
    let mut result = function.clone();

    // Process variables in reverse order for efficiency
    for statement in statements.iter().rev() {
        let var = direct_map[statement];
        let (t_var, f_var) = dual_map[statement];

        // Create the constraint: (state_var => t_var) & (!state_var => f_var)
        // This is equivalent to: (!state_var | t_var) & (state_var | f_var)
        let state_implies_t = Bdd::new_literal(var, false).or(&Bdd::new_literal(t_var, true));
        let not_state_implies_f = Bdd::new_literal(var, true).or(&Bdd::new_literal(f_var, true));
        let is_in_space = state_implies_t.and(&not_state_implies_f);

        // Apply constraint and existentially quantify the state variable
        result = result.and(&is_in_space).exists(&[var]);
    }

    result
}

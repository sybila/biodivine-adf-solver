use ruddy::split::Bdd;

pub mod three_valued;
pub mod two_valued;

pub type DynamicModelSet = Box<dyn ModelSet>;

pub trait ModelSet {
    /// Get a reference to the underlying [`Bdd`].
    fn symbolic_set(&self) -> &Bdd;

    /// Count the models in this set (possibly overflowing to [`f64::INFINITY`]).
    fn model_count(&self) -> f64;
}

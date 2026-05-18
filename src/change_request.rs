use crate::error::PhotoKitError;

/// Wraps `PHChangeRequest`.
pub trait PHChangeRequest: core::fmt::Debug {
    /// Output type returned by `PHChangeRequest`.
    type Output;

    /// Executes the Photos framework change request represented by `PHChangeRequest`.
    fn perform(self) -> Result<Self::Output, PhotoKitError>;
}

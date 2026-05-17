use crate::error::PhotoKitError;

pub trait PHChangeRequest: core::fmt::Debug {
    type Output;

    fn perform(self) -> Result<Self::Output, PhotoKitError>;
}

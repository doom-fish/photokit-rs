use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `CGRect`-style geometry used by PhotosUI and project payloads.
pub struct PHRect {
    /// Horizontal origin of the rectangle.
    pub x: f64,
    /// Vertical origin of the rectangle.
    pub y: f64,
    /// Width of the rectangle.
    pub width: f64,
    /// Height of the rectangle.
    pub height: f64,
}

impl PHRect {
    /// Creates a new rectangle helper for PhotosUI payloads.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Encodes an RGBA color used by PhotosUI payloads.
pub struct PHColor {
    /// Red channel, normalized to `0.0..=1.0`.
    pub red: f64,
    /// Green channel, normalized to `0.0..=1.0`.
    pub green: f64,
    /// Blue channel, normalized to `0.0..=1.0`.
    pub blue: f64,
    /// Alpha channel, normalized to `0.0..=1.0`.
    pub alpha: f64,
}

impl PHColor {
    /// Creates a new color helper for PhotosUI payloads.
    pub const fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_new_sets_fields() {
        assert_eq!(
            PHRect::new(1.0, 2.0, 3.0, 4.0),
            PHRect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }
        );
    }

    #[test]
    fn rect_default_is_zeroed() {
        assert_eq!(PHRect::default(), PHRect::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn color_new_sets_fields() {
        assert_eq!(
            PHColor::new(0.25, 0.5, 0.75, 1.0),
            PHColor {
                red: 0.25,
                green: 0.5,
                blue: 0.75,
                alpha: 1.0,
            }
        );
    }

    #[test]
    fn rect_serde_round_trip_preserves_values() {
        let rect = PHRect::new(10.0, 20.0, 30.0, 40.0);
        let json = serde_json::to_string(&rect).expect("serialize rect");
        let decoded: PHRect = serde_json::from_str(&json).expect("deserialize rect");

        assert_eq!(decoded, rect);
    }

    #[test]
    fn color_serde_round_trip_preserves_values() {
        let color = PHColor::new(0.1, 0.2, 0.3, 0.4);
        let json = serde_json::to_string(&color).expect("serialize color");
        let decoded: PHColor = serde_json::from_str(&json).expect("deserialize color");

        assert_eq!(decoded, color);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// params passed onto the mirror plugin
pub struct MirrorParams {
    /// flip horizontally
    #[serde(default)]
    pub horizontal: bool,
    /// flip vertically
    #[serde(default)]
    pub vertical: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// params passed onto the blur plugin
pub struct BlurParams {
    /// standard deviation of the gaussian kernel
    ///
    /// missing key defaults to 0.0 (still applies a minimal blur, per `image`'s implementation)
    #[serde(default)]
    pub sigma: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// params passed onto the grayscale plugin
///
/// no knobs today, but kept as a struct (rather than accepting any JSON) so
/// the plugin still rejects unrecognized params instead of silently ignoring them
pub struct GrayscaleParams {}

#[cfg(test)]
mod params_tests {
    use super::*;

    #[test]
    fn mirror_params_parses_valid_json() {
        let params: MirrorParams =
            serde_json::from_str(r#"{"horizontal":true,"vertical":false}"#).unwrap();
        assert_eq!(
            params,
            MirrorParams {
                horizontal: true,
                vertical: false
            }
        );
    }

    #[test]
    fn mirror_params_defaults_missing_keys_to_false() {
        let params: MirrorParams = serde_json::from_str("{}").unwrap();
        assert_eq!(
            params,
            MirrorParams {
                horizontal: false,
                vertical: false
            }
        );
    }

    #[test]
    fn mirror_params_rejects_empty_input() {
        assert!(serde_json::from_str::<MirrorParams>("").is_err());
    }

    #[test]
    fn mirror_params_rejects_unknown_keys() {
        assert!(
            serde_json::from_str::<MirrorParams>(r#"{"horizontal":true,"flip":true}"#).is_err()
        );
    }

    #[test]
    fn mirror_params_rejects_non_bool_values() {
        assert!(serde_json::from_str::<MirrorParams>(r#"{"horizontal":"yes"}"#).is_err());
    }

    #[test]
    fn blur_params_parses_valid_json() {
        let params: BlurParams = serde_json::from_str(r#"{"sigma":3.5}"#).unwrap();
        assert_eq!(params, BlurParams { sigma: 3.5 });
    }

    #[test]
    fn blur_params_defaults_sigma_to_zero() {
        let params: BlurParams = serde_json::from_str("{}").unwrap();
        assert_eq!(params, BlurParams { sigma: 0.0 });
    }

    #[test]
    fn blur_params_rejects_empty_input() {
        assert!(serde_json::from_str::<BlurParams>("").is_err());
    }

    #[test]
    fn blur_params_rejects_unknown_keys() {
        assert!(serde_json::from_str::<BlurParams>(r#"{"sigma":3.0,"passes":2}"#).is_err());
    }

    #[test]
    fn blur_params_rejects_non_numeric_values() {
        assert!(serde_json::from_str::<BlurParams>(r#"{"sigma":"three"}"#).is_err());
    }

    #[test]
    fn grayscale_params_parses_empty_object() {
        assert_eq!(
            serde_json::from_str::<GrayscaleParams>("{}").unwrap(),
            GrayscaleParams {}
        );
    }

    #[test]
    fn grayscale_params_rejects_unknown_keys() {
        assert!(serde_json::from_str::<GrayscaleParams>(r#"{"intensity":1}"#).is_err());
    }
}

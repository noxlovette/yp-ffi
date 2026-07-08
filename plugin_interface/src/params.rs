use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MirrorParams {
    #[serde(default)]
    pub horizontal: bool,
    #[serde(default)]
    pub vertical: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlurParams {
    // missing keys default to a no-op blur (radius 0, 0 iterations)
    #[serde(default)]
    pub radius: u32,
    #[serde(default)]
    pub iterations: u32,
}

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
        let params: BlurParams = serde_json::from_str(r#"{"radius":3,"iterations":2}"#).unwrap();
        assert_eq!(
            params,
            BlurParams {
                radius: 3,
                iterations: 2
            }
        );
    }

    #[test]
    fn blur_params_defaults_missing_keys_to_zero() {
        let params: BlurParams = serde_json::from_str("{}").unwrap();
        assert_eq!(
            params,
            BlurParams {
                radius: 0,
                iterations: 0
            }
        );
    }

    #[test]
    fn blur_params_rejects_empty_input() {
        assert!(serde_json::from_str::<BlurParams>("").is_err());
    }

    #[test]
    fn blur_params_rejects_unknown_keys() {
        assert!(serde_json::from_str::<BlurParams>(r#"{"radius":3,"passes":2}"#).is_err());
    }

    #[test]
    fn blur_params_rejects_non_numeric_values() {
        assert!(
            serde_json::from_str::<BlurParams>(r#"{"radius":"three","iterations":2}"#).is_err()
        );
    }
}

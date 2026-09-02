use core::fmt;

use crate::range::Range;
use crate::version::Version;

impl serde::Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VersionVisitor;

        impl serde::de::Visitor<'_> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a semantic version string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

impl serde::Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Range {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RangeVisitor;

        impl serde::de::Visitor<'_> for RangeVisitor {
            type Value = Range;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a semantic version range string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(RangeVisitor)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn version_roundtrip() {
        let version: Version = "1.2.3-alpha.1+build.42".parse().unwrap();
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, "\"1.2.3-alpha.1+build.42\"");
        let parsed: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, version);
    }

    #[test]
    fn range_roundtrip() {
        let range: Range = "^1.2.3 || *".parse().unwrap();
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(json, "\"*\"");
        let parsed: Range = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.to_string(), range.to_string());
    }

    #[test]
    fn deserialize_errors() {
        assert!(serde_json::from_str::<Version>("\"bad\"").is_err());
        assert!(serde_json::from_str::<Range>("\"^\"").is_err());
        assert!(serde_json::from_str::<Version>("123").is_err());
        assert!(serde_json::from_str::<Range>("123").is_err());
    }

    #[test]
    fn version_deserializes_owned_string() {
        let version: Version = serde_json::from_value(serde_json::json!("1.2.3")).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn range_deserializes_owned_string_from_package_json() {
        let package: serde_json::Value =
            serde_json::from_str(r#"{"dependencies":{"react":"^19.0.0"}}"#).unwrap();
        let range: Range =
            serde_json::from_value(package["dependencies"]["react"].clone()).unwrap();
        assert!(range.satisfies(&Version::new(19, 1, 0)));
    }

    #[test]
    fn version_and_range_deserialize_transient_strings() {
        let version: Version = serde_json::from_str(r#""1.2.\u0033""#).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));

        let range: Range = serde_json::from_str(r#""^19.\u0030.0""#).unwrap();
        assert!(range.satisfies(&Version::new(19, 1, 0)));
    }
}

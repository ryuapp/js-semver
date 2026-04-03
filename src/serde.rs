use crate::range::Range;
use crate::version::Version;

impl serde::Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for Range {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Range {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
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
}

//! Integer tool parameters: some MCP clients send a JSON number, others send the same value
//! as a string (e.g. `"3"` instead of `3`).
//!
//! Plain `Option<i32>` rejects the string form with
//! `invalid type: string "3", expected i32`, which surfaces to the user as a hard tool
//! failure for an otherwise well-formed request. This mirrors the approach `FlexibleTagList`
//! takes for `tags`, accepting either wire shape and normalising to `i32`.

use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor};
use serde::ser::{Serialize, Serializer};

/// An `i32` from the wire: a native JSON number, or a string that parses as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlexibleI32(pub i32);

impl FlexibleI32 {
    pub fn value(self) -> i32 {
        self.0
    }
}

impl From<FlexibleI32> for i32 {
    fn from(value: FlexibleI32) -> Self {
        value.0
    }
}

impl Serialize for FlexibleI32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FlexibleI32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlexibleI32Visitor;

        impl<'de> Visitor<'de> for FlexibleI32Visitor {
            type Value = FlexibleI32;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an integer, or a string containing an integer")
            }

            fn visit_i64<E: DeError>(self, v: i64) -> Result<Self::Value, E> {
                i32::try_from(v)
                    .map(FlexibleI32)
                    .map_err(|_| E::custom(format!("integer {v} is out of range for i32")))
            }

            fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
                i32::try_from(v)
                    .map(FlexibleI32)
                    .map_err(|_| E::custom(format!("integer {v} is out of range for i32")))
            }

            fn visit_f64<E: DeError>(self, v: f64) -> Result<Self::Value, E> {
                // Accept whole-valued floats (JSON has no integer type, so 3.0 may arrive
                // where 3 was meant), but reject genuine fractions rather than truncating.
                if v.fract() == 0.0 && v >= i32::MIN as f64 && v <= i32::MAX as f64 {
                    Ok(FlexibleI32(v as i32))
                } else {
                    Err(E::custom(format!("expected a whole number, got {v}")))
                }
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                let trimmed = v.trim();
                trimmed.parse::<i32>().map(FlexibleI32).map_err(|_| {
                    E::custom(format!("string \"{v}\" does not contain a valid integer"))
                })
            }
        }

        deserializer.deserialize_any(FlexibleI32Visitor)
    }
}

impl JsonSchema for FlexibleI32 {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FlexibleI32".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        // Advertise both accepted wire shapes so clients can see the string form is allowed.
        serde_json::from_value(serde_json::json!({
            "anyOf": [
                { "type": "integer", "format": "int32", "description": "Integer (preferred)." },
                { "type": "string", "description": "Integer encoded as a string, e.g. \"3\"." }
            ]
        }))
        .expect("FlexibleI32 schema is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<FlexibleI32, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn accepts_native_integer() {
        assert_eq!(parse("3").unwrap(), FlexibleI32(3));
    }

    #[test]
    fn accepts_integer_encoded_as_string() {
        assert_eq!(parse("\"3\"").unwrap(), FlexibleI32(3));
    }

    #[test]
    fn accepts_negative_and_whitespace_padded_strings() {
        assert_eq!(parse("\"-12\"").unwrap(), FlexibleI32(-12));
        assert_eq!(parse("\" 7 \"").unwrap(), FlexibleI32(7));
    }

    #[test]
    fn accepts_whole_valued_float() {
        assert_eq!(parse("3.0").unwrap(), FlexibleI32(3));
    }

    #[test]
    fn rejects_fractional_values_rather_than_truncating() {
        assert!(parse("3.5").is_err());
    }

    #[test]
    fn rejects_non_numeric_strings() {
        assert!(parse("\"three\"").is_err());
        assert!(parse("\"\"").is_err());
    }

    #[test]
    fn rejects_out_of_range_integers() {
        assert!(parse("9999999999").is_err());
    }

    #[test]
    fn round_trips_back_to_a_plain_number() {
        let value: FlexibleI32 = parse("\"42\"").unwrap();
        assert_eq!(serde_json::to_string(&value).unwrap(), "42");
    }
}

// src/core/builder/serde_ops.rs
// JSON constructors for QueryBuilder — enabled by `feature = "serde"`.

#[cfg(feature = "serde")]
use super::{QueryBuilder, QueryData};
#[cfg(feature = "serde")]
use crate::core::error::{RinqError, RinqResult};
#[cfg(feature = "serde")]
use crate::core::state::Initial;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
#[cfg(feature = "serde")]
use std::marker::PhantomData;

/// Construct a `QueryBuilder` by deserialising a JSON array string.
///
/// Available when the `serde` feature is enabled.
#[cfg(feature = "serde")]
impl<T: DeserializeOwned + 'static> QueryBuilder<T, Initial> {
    /// Parse a JSON array string into a `QueryBuilder<T, Initial>`.
    ///
    /// The JSON must be a top-level array (`[…]`). Each element is
    /// deserialised into `T`. Returns `Err` if parsing or deserialisation
    /// fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")] {
    /// use rinq::QueryBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let json = r#"[{"x":1,"y":2},{"x":3,"y":4}]"#;
    /// let result: Vec<Point> = QueryBuilder::<Point, _>::from_json(json)
    ///     .unwrap()
    ///     .where_(|p| p.x > 1)
    ///     .collect();
    /// assert_eq!(result, vec![Point { x: 3, y: 4 }]);
    /// # }
    /// ```
    pub fn from_json(json: &str) -> RinqResult<Self> {
        let items: Vec<T> = serde_json::from_str(json).map_err(|e| RinqError::ExecutionError {
            message: e.to_string(),
        })?;
        Ok(QueryBuilder {
            data: QueryData::Iterator(Box::new(items.into_iter())),
            _state: PhantomData,
        })
    }
}

/// Dynamic JSON query using `serde_json::Value` — no schema required.
///
/// Available when the `serde` feature is enabled.
#[cfg(feature = "serde")]
impl QueryBuilder<serde_json::Value, Initial> {
    /// Parse a JSON array string into a `QueryBuilder<serde_json::Value, Initial>`.
    ///
    /// Useful when the schema is not known at compile time or when you want to
    /// access fields dynamically via index notation (`v["field"]`).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")] {
    /// use rinq::QueryBuilder;
    ///
    /// let json = r#"[{"age":30},{"age":17},{"age":25}]"#;
    /// let adults: Vec<_> = QueryBuilder::from_json_value(json)
    ///     .unwrap()
    ///     .where_(|v| v["age"].as_u64().unwrap_or(0) >= 18)
    ///     .collect();
    /// assert_eq!(adults.len(), 2);
    /// # }
    /// ```
    pub fn from_json_value(json: &str) -> RinqResult<Self> {
        Self::from_json(json)
    }
}

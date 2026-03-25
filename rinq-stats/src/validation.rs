// rinq-stats/src/validation.rs
// Phase B4: ValidationExt trait — chainable validation pipeline for QueryBuilder.

use rinq::QueryBuilder;

/// A single validation failure.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Name of the rule that failed (the `rule` label passed to `validate()`).
    pub rule: String,
    /// Human-readable message describing the violation.
    pub message: String,
    /// Zero-based index of the element that triggered this error.
    pub index: usize,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} (index {})", self.rule, self.message, self.index)
    }
}

/// A builder that chains validation rules over a sequence of items.
///
/// Construct one via [`ValidationExt::validate`] on any [`QueryBuilder`].
///
/// # Example
///
/// ```
/// use rinq::QueryBuilder;
/// use rinq_stats::ValidationExt;
///
/// let result = QueryBuilder::from(vec![1_i32, -2, 3, -4])
///     .validate(|x| *x > 0, "positive", "must be positive")
///     .collect_validated();
///
/// let errors = result.unwrap_err();
/// assert_eq!(errors.len(), 2);
/// assert_eq!(errors[0].index, 1);
/// assert_eq!(errors[1].index, 3);
/// ```
pub struct ValidationQueryBuilder<T> {
    items: Vec<T>,
    #[allow(clippy::type_complexity)]
    rules: Vec<(String, String, Box<dyn Fn(&T) -> bool>)>,
}

impl<T> ValidationQueryBuilder<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items, rules: Vec::new() }
    }

    /// Add another validation rule.
    ///
    /// - `rule`: short identifier used in [`ValidationError::rule`].
    /// - `message`: human-readable error text used in [`ValidationError::message`].
    /// - `predicate`: returns `true` when the item is *valid*.
    pub fn validate<F>(mut self, predicate: F, rule: &str, message: &str) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        self.rules.push((rule.to_owned(), message.to_owned(), Box::new(predicate)));
        self
    }

    /// Run all validation rules over every element.
    ///
    /// Returns `Ok(Vec<T>)` when every item passes every rule.
    /// Returns `Err(Vec<ValidationError>)` (non-empty) when at least one item
    /// fails at least one rule — **all** violations are collected, not just the first.
    pub fn collect_validated(self) -> Result<Vec<T>, Vec<ValidationError>> {
        let mut errors: Vec<ValidationError> = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            for (rule, message, predicate) in &self.rules {
                if !predicate(item) {
                    errors.push(ValidationError {
                        rule: rule.clone(),
                        message: message.clone(),
                        index,
                    });
                }
            }
        }
        if errors.is_empty() {
            Ok(self.items)
        } else {
            Err(errors)
        }
    }

    /// Return only the items that pass all validation rules,
    /// discarding any that fail at least one rule.
    pub fn collect_valid(self) -> Vec<T> {
        self.items
            .into_iter()
            .filter(|item| self.rules.iter().all(|(_, _, pred)| pred(item)))
            .collect()
    }

    /// Return only the items that fail at least one validation rule,
    /// paired with their errors.
    pub fn collect_invalid(self) -> Vec<(T, Vec<ValidationError>)> {
        self.items
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let item_errors: Vec<ValidationError> = self
                    .rules
                    .iter()
                    .filter(|(_, _, pred)| !pred(&item))
                    .map(|(rule, message, _)| ValidationError {
                        rule: rule.clone(),
                        message: message.clone(),
                        index,
                    })
                    .collect();
                if item_errors.is_empty() {
                    None
                } else {
                    Some((item, item_errors))
                }
            })
            .collect()
    }
}

// ── ValidationExt trait ───────────────────────────────────────────────────────

/// Validation operations for [`QueryBuilder`].
///
/// Import this trait to add chainable `.validate()` to any query builder:
///
/// ```
/// use rinq::QueryBuilder;
/// use rinq_stats::ValidationExt;
///
/// let result = QueryBuilder::from(vec![5_i32, -1, 3])
///     .validate(|x| *x > 0, "positive", "value must be positive")
///     .collect_validated();
///
/// assert!(result.is_err());
/// let errors = result.unwrap_err();
/// assert_eq!(errors[0].index, 1);
/// ```
pub trait ValidationExt<T> {
    /// Begin a validation pipeline.
    ///
    /// `predicate` returns `true` when the item is *valid*.
    /// `rule` is a short identifier; `message` is the human-readable error text.
    ///
    /// Chain additional rules with [`ValidationQueryBuilder::validate`].
    fn validate<F>(self, predicate: F, rule: &str, message: &str) -> ValidationQueryBuilder<T>
    where
        F: Fn(&T) -> bool + 'static;
}

impl<T: 'static, State: 'static> ValidationExt<T> for QueryBuilder<T, State> {
    fn validate<F>(self, predicate: F, rule: &str, message: &str) -> ValidationQueryBuilder<T>
    where
        F: Fn(&T) -> bool + 'static,
    {
        let items: Vec<T> = self.collect();
        let mut vqb = ValidationQueryBuilder::new(items);
        vqb = vqb.validate(predicate, rule, message);
        vqb
    }
}

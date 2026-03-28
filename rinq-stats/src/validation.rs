// rinq-stats/src/validation.rs
// Phase B4: ValidationExt + Phase 4G I3: validate_if / validate_with extensions.

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

type CheckFn<T> = Box<dyn Fn(&T) -> Option<String>>;

struct ValidationRule<T> {
    rule: String,
    /// Returns `Some(error_message)` when the item *fails* validation,
    /// or `None` when the item *passes*.
    check: CheckFn<T>,
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
    rules: Vec<ValidationRule<T>>,
}

impl<T> ValidationQueryBuilder<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items, rules: Vec::new() }
    }

    /// Add another validation rule with a fixed error message.
    ///
    /// - `rule`: short identifier used in [`ValidationError::rule`].
    /// - `message`: human-readable error text used in [`ValidationError::message`].
    /// - `predicate`: returns `true` when the item is *valid*.
    pub fn validate<F>(mut self, predicate: F, rule: &str, message: &str) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        let msg = message.to_owned();
        self.rules.push(ValidationRule {
            rule: rule.to_owned(),
            check: Box::new(move |item| if predicate(item) { None } else { Some(msg.clone()) }),
        });
        self
    }

    /// Add a validation rule that only runs when `condition` is `true`.
    ///
    /// When `condition` is `false` the rule is silently skipped for every item,
    /// as if it had never been registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::ValidationExt;
    ///
    /// // condition=false → the negative-check is never applied
    /// let result = QueryBuilder::from(vec![-1_i32, -2])
    ///     .validate(|_| true, "always_ok", "dummy")
    ///     .validate_if(false, |x| *x > 0, "positive", "must be positive")
    ///     .collect_validated();
    /// assert!(result.is_ok());
    /// ```
    pub fn validate_if<F>(mut self, condition: bool, predicate: F, rule: &str, message: &str) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        if condition {
            let msg = message.to_owned();
            self.rules.push(ValidationRule {
                rule: rule.to_owned(),
                check: Box::new(move |item| {
                    if predicate(item) { None } else { Some(msg.clone()) }
                }),
            });
        }
        self
    }

    /// Add a validation rule whose error message is generated dynamically.
    ///
    /// The closure `make_message` receives the violating item and returns an
    /// owned `String` describing the specific violation.  This is useful when
    /// the error text needs to include the item's value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::ValidationExt;
    ///
    /// let result = QueryBuilder::from(vec![1_i32, -5, 3])
    ///     .validate(|_| true, "dummy", "")
    ///     .validate_with(|x| *x > 0, "positive", |x| format!("{x} is not positive"))
    ///     .collect_validated();
    /// let errors = result.unwrap_err();
    /// assert_eq!(errors[0].message, "-5 is not positive");
    /// ```
    pub fn validate_with<F, M>(mut self, predicate: F, rule: &str, make_message: M) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        M: Fn(&T) -> String + 'static,
    {
        self.rules.push(ValidationRule {
            rule: rule.to_owned(),
            check: Box::new(move |item| {
                if predicate(item) { None } else { Some(make_message(item)) }
            }),
        });
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
            for r in &self.rules {
                if let Some(message) = (r.check)(item) {
                    errors.push(ValidationError { rule: r.rule.clone(), message, index });
                }
            }
        }
        if errors.is_empty() { Ok(self.items) } else { Err(errors) }
    }

    /// Return only the items that pass all validation rules,
    /// discarding any that fail at least one rule.
    pub fn collect_valid(self) -> Vec<T> {
        self.items
            .into_iter()
            .filter(|item| self.rules.iter().all(|r| (r.check)(item).is_none()))
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
                    .filter_map(|r| {
                        (r.check)(&item).map(|msg| ValidationError {
                            rule: r.rule.clone(),
                            message: msg,
                            index,
                        })
                    })
                    .collect();
                if item_errors.is_empty() { None } else { Some((item, item_errors)) }
            })
            .collect()
    }

    /// Add a range-check rule: the field extracted by `field_fn` must satisfy `min <= v <= max`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::ValidationExt;
    ///
    /// let result = QueryBuilder::from(vec![5_i32, 150, 30])
    ///     .validate(|_| true, "dummy", "")
    ///     .validate_range(|x| *x, 0, 100, "in_range");
    /// let errors = result.collect_validated().unwrap_err();
    /// assert_eq!(errors.len(), 1);
    /// assert_eq!(errors[0].rule, "in_range");
    /// ```
    pub fn validate_range<F, N>(mut self, field_fn: F, min: N, max: N, rule: &str) -> Self
    where
        F: Fn(&T) -> N + 'static,
        N: PartialOrd + std::fmt::Display + Copy + 'static,
    {
        let rule_str = rule.to_owned();
        self.rules.push(ValidationRule {
            rule: rule_str,
            check: Box::new(move |item| {
                let v = field_fn(item);
                if v >= min && v <= max {
                    None
                } else {
                    Some(format!("value {v} is out of range [{min}, {max}]"))
                }
            }),
        });
        self
    }

    /// Add a uniqueness rule: the key extracted by `key_fn` must not appear more than once.
    ///
    /// Uses an internal `RefCell<HashSet>` so duplicates after the first occurrence are flagged.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::ValidationExt;
    ///
    /// let result = QueryBuilder::from(vec![1_i32, 2, 1, 3])
    ///     .validate(|_| true, "dummy", "")
    ///     .validate_unique(|x| *x, "unique_id");
    /// let errors = result.collect_validated().unwrap_err();
    /// assert_eq!(errors.len(), 1); // second occurrence of 1
    /// assert_eq!(errors[0].index, 2);
    /// ```
    pub fn validate_unique<F, K>(mut self, key_fn: F, rule: &str) -> Self
    where
        F: Fn(&T) -> K + 'static,
        K: std::hash::Hash + Eq + 'static,
    {
        use std::cell::RefCell;
        use std::collections::HashSet;
        let seen: RefCell<HashSet<K>> = RefCell::new(HashSet::new());
        let rule_str = rule.to_owned();
        self.rules.push(ValidationRule {
            rule: rule_str,
            check: Box::new(move |item| {
                let k = key_fn(item);
                let mut guard = seen.borrow_mut();
                if guard.contains(&k) {
                    Some("duplicate value detected".to_owned())
                } else {
                    guard.insert(k);
                    None
                }
            }),
        });
        self
    }

    /// Add a non-empty check for a `&str` field: the field must not be an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::ValidationExt;
    ///
    /// #[derive(Clone, Debug)]
    /// struct User { name: String }
    ///
    /// let result = QueryBuilder::from(vec![
    ///     User { name: "Alice".to_owned() },
    ///     User { name: "".to_owned() },
    /// ])
    /// .validate(|_| true, "dummy", "")
    /// .validate_non_empty(|u| u.name.as_str(), "name_required");
    /// let errors = result.collect_validated().unwrap_err();
    /// assert_eq!(errors.len(), 1);
    /// assert_eq!(errors[0].index, 1);
    /// ```
    pub fn validate_non_empty<F>(mut self, field_fn: F, rule: &str) -> Self
    where
        F: Fn(&T) -> &str + 'static,
    {
        let rule_str = rule.to_owned();
        self.rules.push(ValidationRule {
            rule: rule_str,
            check: Box::new(move |item| {
                if field_fn(item).is_empty() {
                    Some("field must not be empty".to_owned())
                } else {
                    None
                }
            }),
        });
        self
    }

    /// Run all validation rules and return a `Vec<String>` of formatted error messages.
    ///
    /// Returns an empty `Vec` when there are no violations.
    /// Each string has the format produced by [`ValidationError`]'s `Display` impl:
    /// `"[rule] message (index N)"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rinq::QueryBuilder;
    /// use rinq_stats::ValidationExt;
    ///
    /// let report = QueryBuilder::from(vec![1_i32, -2, 3])
    ///     .validate(|x| *x > 0, "positive", "must be positive")
    ///     .report();
    /// assert_eq!(report.len(), 1);
    /// assert!(report[0].contains("positive"));
    /// ```
    pub fn report(self) -> Vec<String> {
        match self.collect_validated() {
            Ok(_) => vec![],
            Err(errors) => errors.iter().map(|e| e.to_string()).collect(),
        }
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

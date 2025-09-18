use sqlx::{Arguments, Postgres};
use std::collections::VecDeque;

/// Enhanced parameter binding system with proper type handling
#[derive(Debug, Clone)]
pub struct ParameterBinder {
    parameters: VecDeque<SqlValue>,
}

#[derive(Debug, Clone)]
pub enum SqlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl SqlValue {
    pub fn from_string(s: &str) -> Self {
        // Try to parse as different types
        if let Ok(int_val) = s.parse::<i64>() {
            Self::Integer(int_val)
        } else if let Ok(float_val) = s.parse::<f64>() {
            Self::Float(float_val)
        } else if let Ok(bool_val) = s.parse::<bool>() {
            Self::Boolean(bool_val)
        } else if s.eq_ignore_ascii_case("null") {
            Self::Null
        } else {
            Self::String(s.to_string())
        }
    }

    pub fn bind_to_arguments(&self, args: &mut sqlx::postgres::PgArguments) {
        match self {
            SqlValue::String(s) => {
                let _ = args.add(s);
            },
            SqlValue::Integer(i) => {
                let _ = args.add(i);
            },
            SqlValue::Float(f) => {
                let _ = args.add(f);
            },
            SqlValue::Boolean(b) => {
                let _ = args.add(b);
            },
            SqlValue::Null => {
                let _ = args.add(None::<String>);
            },
        }
    }
}

impl ParameterBinder {
    pub fn new() -> Self {
        Self {
            parameters: VecDeque::new(),
        }
    }

    pub fn add_parameter(&mut self, value: SqlValue) -> String {
        self.parameters.push_back(value);
        format!("${}", self.parameters.len())
    }

    pub fn add_string(&mut self, value: String) -> String {
        self.add_parameter(SqlValue::String(value))
    }

    pub fn add_smart_parameter(&mut self, value: String) -> String {
        self.add_parameter(SqlValue::from_string(&value))
    }

    pub fn add_parameters(&mut self, values: Vec<String>) -> Vec<String> {
        values.into_iter()
            .map(|v| self.add_smart_parameter(v))
            .collect()
    }

    pub fn bind_all<'q>(&'q self, mut query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
        for param in &self.parameters {
            match param {
                SqlValue::String(s) => { query = query.bind(s); },
                SqlValue::Integer(i) => { query = query.bind(i); },
                SqlValue::Float(f) => { query = query.bind(f); },
                SqlValue::Boolean(b) => { query = query.bind(b); },
                SqlValue::Null => { query = query.bind(None::<String>); },
            }
        }
        query
    }

    pub fn bind_all_as<'q, T>(&'q self, mut query: sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>) -> sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        for param in &self.parameters {
            match param {
                SqlValue::String(s) => { query = query.bind(s); },
                SqlValue::Integer(i) => { query = query.bind(i); },
                SqlValue::Float(f) => { query = query.bind(f); },
                SqlValue::Boolean(b) => { query = query.bind(b); },
                SqlValue::Null => { query = query.bind(None::<String>); },
            }
        }
        query
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    pub fn clear(&mut self) {
        self.parameters.clear();
    }
}

impl Default for ParameterBinder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for improved parameter binding
pub trait QueryBinderExt<'q> {
    fn bind_all_parameters(self, args: sqlx::postgres::PgArguments) -> Self;
}

impl<'q> QueryBinderExt<'q> for sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    fn bind_all_parameters(self, _args: sqlx::postgres::PgArguments) -> Self {
        // SQLx doesn't provide direct access to replace arguments in a query
        // This trait is kept for compatibility but the SmartBind trait should be used instead
        self
    }
}

impl<'q, T> QueryBinderExt<'q> for sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    fn bind_all_parameters(self, _args: sqlx::postgres::PgArguments) -> Self {
        // SQLx doesn't provide direct access to replace arguments in a query
        // This trait is kept for compatibility but the SmartBind trait should be used instead
        self
    }
}

/// Improved version of the BindAll trait with better type safety
pub trait SmartBind<'q> {
    fn bind_smart_params(self, binder: &'q ParameterBinder) -> Self;
}

impl<'q> SmartBind<'q> for sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    fn bind_smart_params(mut self, binder: &'q ParameterBinder) -> Self {
        for param in &binder.parameters {
            match param {
                SqlValue::String(s) => { self = self.bind(s); },
                SqlValue::Integer(i) => { self = self.bind(i); },
                SqlValue::Float(f) => { self = self.bind(f); },
                SqlValue::Boolean(b) => { self = self.bind(b); },
                SqlValue::Null => { self = self.bind(None::<String>); },
            }
        }
        self
    }
}

impl<'q, T> SmartBind<'q> for sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    fn bind_smart_params(mut self, binder: &'q ParameterBinder) -> Self {
        for param in &binder.parameters {
            match param {
                SqlValue::String(s) => { self = self.bind(s); },
                SqlValue::Integer(i) => { self = self.bind(i); },
                SqlValue::Float(f) => { self = self.bind(f); },
                SqlValue::Boolean(b) => { self = self.bind(b); },
                SqlValue::Null => { self = self.bind(None::<String>); },
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_binder() {
        let mut binder = ParameterBinder::new();

        let param1 = binder.add_string("test".to_string());
        let param2 = binder.add_smart_parameter("42".to_string());
        let param3 = binder.add_smart_parameter("true".to_string());

        assert_eq!(param1, "$1");
        assert_eq!(param2, "$2");
        assert_eq!(param3, "$3");
        assert_eq!(binder.parameter_count(), 3);
    }

    #[test]
    fn test_sql_value_parsing() {
        assert!(matches!(SqlValue::from_string("42"), SqlValue::Integer(42)));
        assert!(matches!(SqlValue::from_string("3.14"), SqlValue::Float(_)));
        assert!(matches!(SqlValue::from_string("true"), SqlValue::Boolean(true)));
        assert!(matches!(SqlValue::from_string("null"), SqlValue::Null));
        assert!(matches!(SqlValue::from_string("text"), SqlValue::String(_)));
    }

    #[test]
    fn test_multiple_parameters() {
        let mut binder = ParameterBinder::new();
        let params = binder.add_parameters(vec!["1".to_string(), "2".to_string(), "3".to_string()]);

        assert_eq!(params, vec!["$1", "$2", "$3"]);
        assert_eq!(binder.parameter_count(), 3);
    }
}
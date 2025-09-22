use bigdecimal::BigDecimal;
use diesel::{AsExpression, FromSqlRow};
use diesel::sql_types::Numeric;
use serde::{Deserialize, Serialize};

/// Wrapper around BigDecimal that implements ToSchema for OpenAPI
/// Serializes as string to avoid floating point precision issues
#[derive(Debug, Clone, PartialEq, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = Numeric)]
pub struct DecimalWrapper(pub BigDecimal);

// For ToSchema, just represent it as a string in OpenAPI docs
impl utoipa::PartialSchema for DecimalWrapper {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::Schema::Object(
            utoipa::openapi::schema::ObjectBuilder::new()
                .schema_type(utoipa::openapi::schema::SchemaType::new(utoipa::openapi::schema::Type::String))
                .description(Some("Decimal number represented as string"))
                .examples(Some(serde_json::Value::String("123.45".to_string())))
                .build()
        ).into()
    }
}

impl utoipa::ToSchema for DecimalWrapper {
    fn name() -> std::borrow::Cow<'static, str> {
        "DecimalWrapper".into()
    }
}

impl From<BigDecimal> for DecimalWrapper {
    fn from(decimal: BigDecimal) -> Self {
        DecimalWrapper(decimal)
    }
}

impl From<DecimalWrapper> for BigDecimal {
    fn from(wrapper: DecimalWrapper) -> Self {
        wrapper.0
    }
}

impl From<i32> for DecimalWrapper {
    fn from(val: i32) -> Self {
        DecimalWrapper(BigDecimal::from(val))
    }
}

impl From<i64> for DecimalWrapper {
    fn from(val: i64) -> Self {
        DecimalWrapper(BigDecimal::from(val))
    }
}

impl From<u32> for DecimalWrapper {
    fn from(val: u32) -> Self {
        DecimalWrapper(BigDecimal::from(val))
    }
}

impl From<u64> for DecimalWrapper {
    fn from(val: u64) -> Self {
        DecimalWrapper(BigDecimal::from(val))
    }
}

impl std::ops::Deref for DecimalWrapper {
    type Target = BigDecimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DecimalWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Implement diesel traits
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::pg::Pg;

impl FromSql<Numeric, Pg> for DecimalWrapper {
    fn from_sql(bytes: diesel::pg::PgValue) -> deserialize::Result<Self> {
        let decimal = BigDecimal::from_sql(bytes)?;
        Ok(DecimalWrapper(decimal))
    }
}

impl ToSql<Numeric, Pg> for DecimalWrapper {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <BigDecimal as ToSql<Numeric, Pg>>::to_sql(&self.0, out)
    }
}
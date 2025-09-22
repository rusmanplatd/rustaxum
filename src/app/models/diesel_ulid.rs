use diesel::sql_types::Text;
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql, IsNull};
use diesel::deserialize::{self, FromSql};
use diesel::AsExpression;
use diesel::FromSqlRow;
use diesel::pg::PgValue;
use ulid::Ulid;
use serde::{Deserialize, Serialize};
use std::io::Write;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsExpression, FromSqlRow, Serialize, Deserialize, ToSchema)]
#[diesel(sql_type = Text)]
pub struct DieselUlid(pub Ulid);

impl DieselUlid {
    pub fn new() -> Self {
        DieselUlid(Ulid::new())
    }

    pub fn from_string(s: &str) -> Result<Self, ulid::DecodeError> {
        Ok(DieselUlid(Ulid::from_string(s)?))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn inner(&self) -> Ulid {
        self.0
    }
}

impl Default for DieselUlid {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Ulid> for DieselUlid {
    fn from(ulid: Ulid) -> Self {
        DieselUlid(ulid)
    }
}

impl From<DieselUlid> for Ulid {
    fn from(diesel_ulid: DieselUlid) -> Self {
        diesel_ulid.0
    }
}

impl From<DieselUlid> for String {
    fn from(diesel_ulid: DieselUlid) -> Self {
        diesel_ulid.to_string()
    }
}

impl std::fmt::Display for DieselUlid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSql<Text, Pg> for DieselUlid {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.0.to_string().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for DieselUlid {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        Ok(DieselUlid(Ulid::from_string(&s)?))
    }
}


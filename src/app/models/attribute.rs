use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl AttributeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttributeType::String => "string",
            AttributeType::Number => "number",
            AttributeType::Boolean => "boolean",
            AttributeType::Array => "array",
            AttributeType::Object => "object",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "string" => Some(AttributeType::String),
            "number" => Some(AttributeType::Number),
            "boolean" => Some(AttributeType::Boolean),
            "array" => Some(AttributeType::Array),
            "object" => Some(AttributeType::Object),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: Ulid,
    pub name: String,
    pub attribute_type: AttributeType,
    pub value: Value,
    pub subject_type: String,
    pub subject_id: Option<Ulid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Ulid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAttribute {
    pub name: String,
    pub attribute_type: AttributeType,
    pub value: Value,
    pub subject_type: String,
    pub subject_id: Option<Ulid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Ulid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAttribute {
    pub name: Option<String>,
    pub attribute_type: Option<AttributeType>,
    pub value: Option<Value>,
    pub subject_type: Option<String>,
    pub subject_id: Option<Ulid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Ulid>,
}

#[derive(Debug, Serialize)]
pub struct AttributeResponse {
    pub id: String,
    pub name: String,
    pub attribute_type: AttributeType,
    pub value: Value,
    pub subject_type: String,
    pub subject_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Attribute {
    pub fn new(
        name: String,
        attribute_type: AttributeType,
        value: Value,
        subject_type: String,
        subject_id: Option<Ulid>,
        resource_type: Option<String>,
        resource_id: Option<Ulid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            attribute_type,
            value,
            subject_type,
            subject_id,
            resource_type,
            resource_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> AttributeResponse {
        AttributeResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            attribute_type: self.attribute_type.clone(),
            value: self.value.clone(),
            subject_type: self.subject_type.clone(),
            subject_id: self.subject_id.map(|id| id.to_string()),
            resource_type: self.resource_type.clone(),
            resource_id: self.resource_id.map(|id| id.to_string()),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Attribute {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let attribute_type_str: String = row.try_get("attribute_type")?;
        let attribute_type = AttributeType::from_str(&attribute_type_str)
            .ok_or_else(|| sqlx::Error::ColumnDecode {
                index: "attribute_type".to_string(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid attribute type",
                )),
            })?;

        let subject_id = if let Ok(subject_id_str) = row.try_get::<String, _>("subject_id") {
            Some(Ulid::from_string(&subject_id_str).map_err(|e| sqlx::Error::ColumnDecode {
                index: "subject_id".to_string(),
                source: Box::new(e),
            })?)
        } else {
            None
        };

        let resource_id = if let Ok(resource_id_str) = row.try_get::<String, _>("resource_id") {
            Some(Ulid::from_string(&resource_id_str).map_err(|e| sqlx::Error::ColumnDecode {
                index: "resource_id".to_string(),
                source: Box::new(e),
            })?)
        } else {
            None
        };

        Ok(Attribute {
            id,
            name: row.try_get("name")?,
            attribute_type,
            value: row.try_get("value")?,
            subject_type: row.try_get("subject_type")?,
            subject_id,
            resource_type: row.try_get("resource_type")?,
            resource_id,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

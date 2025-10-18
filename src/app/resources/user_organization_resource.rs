use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use crate::app::models::user_organization::UserOrganization;

/// User Organization Resource for API responses
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserOrganizationResource {
    /// Unique identifier of the user organization relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User ID associated with this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub user_id: String,
    /// Organization ID associated with this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: String,
    /// Organization position ID for this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_id: String,
    /// Whether this relationship is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// When this relationship started
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub started_at: DateTime<Utc>,
    /// When this relationship ended (if applicable)
    #[schema(example = "2024-12-31T23:59:59Z")]
    pub ended_at: Option<DateTime<Utc>>,
    /// When this record was created
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// When this record was last updated
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

impl UserOrganizationResource {
    /// Create a resource from a UserOrganization model
    pub fn from_model(model: UserOrganization) -> Self {
        Self {
            id: model.id.to_string(),
            user_id: model.user_id.to_string(),
            organization_id: model.organization_id.to_string(),
            organization_position_id: model.organization_position_id.to_string(),
            is_active: model.is_active,
            started_at: model.started_at,
            ended_at: model.ended_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    /// Create a collection of resources from a vector of UserOrganization models
    pub fn collection(models: Vec<UserOrganization>) -> Vec<Self> {
        models.into_iter().map(Self::from_model).collect()
    }
}

/// Extended User Organization Resource with related data
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserOrganizationResourceWithRelations {
    /// Unique identifier of the user organization relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User ID associated with this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub user_id: String,
    /// Organization ID associated with this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: String,
    /// Organization position ID for this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_id: String,
    /// Whether this relationship is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// When this relationship started
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub started_at: DateTime<Utc>,
    /// When this relationship ended (if applicable)
    #[schema(example = "2024-12-31T23:59:59Z")]
    pub ended_at: Option<DateTime<Utc>>,
    /// When this record was created
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// When this record was last updated
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// User information
    pub user: Option<UserBasicInfo>,
    /// Organization information
    pub organization: Option<OrganizationBasicInfo>,
    /// Organization position information
    pub organization_position: Option<OrganizationPositionBasicInfo>,
    /// Roles assigned to this user organization relationship
    pub roles: Vec<RoleBasicInfo>,
    /// Permissions assigned to this user organization relationship (direct and from roles)
    pub permissions: Vec<PermissionBasicInfo>,
}

/// Basic user information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserBasicInfo {
    /// User ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User email
    #[schema(example = "john@example.com")]
    pub email: String,
}

/// Basic organization information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationBasicInfo {
    /// Organization ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Organization name
    #[schema(example = "Engineering Department")]
    pub name: String,
    /// Organization code
    #[schema(example = "ENG-001")]
    pub code: Option<String>,
    /// Organization type information
    pub organization_type: Option<OrganizationTypeBasicInfo>,
    /// Organization domain information
    pub organization_domain: Option<OrganizationDomainBasicInfo>,
}

/// Basic organization type information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationTypeBasicInfo {
    /// Organization type ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Organization type name
    #[schema(example = "Ministry")]
    pub name: String,
    /// Organization type code
    #[schema(example = "MIN")]
    pub code: Option<String>,
    /// Hierarchical level
    #[schema(example = 1)]
    pub level: i32,
}

/// Basic organization domain information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationDomainBasicInfo {
    /// Organization domain ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Organization domain name
    #[schema(example = "Government")]
    pub name: String,
    /// Organization domain code
    #[schema(example = "GOV")]
    pub code: Option<String>,
}

/// Basic organization position information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationPositionBasicInfo {
    /// Organization position ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Organization position name
    #[schema(example = "Senior Software Engineer")]
    pub name: String,
    /// Organization position code
    #[schema(example = "SSE-001")]
    pub code: Option<String>,
    /// Job level
    pub organization_position_level: Option<OrganizationPositionLevelBasicInfo>,
}

/// Basic organization position level information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationPositionLevelBasicInfo {
    /// Job level ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Job level name
    #[schema(example = "Senior Level")]
    pub name: String,
    /// Job level numeric value
    #[schema(example = 5)]
    pub level: i32,
}

/// Basic role information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RoleBasicInfo {
    /// Role ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Role name
    #[schema(example = "Manager")]
    pub name: String,
    /// Role description
    #[schema(example = "Department manager role")]
    pub description: Option<String>,
    /// Role guard name
    #[schema(example = "web")]
    pub guard_name: String,
}

/// Basic permission information for relations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PermissionBasicInfo {
    /// Permission ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Permission name
    #[schema(example = "edit_posts")]
    pub name: String,
    /// Resource this permission applies to
    #[schema(example = "posts")]
    pub resource: Option<String>,
    /// Action this permission allows
    #[schema(example = "edit")]
    pub action: String,
    /// Permission guard name
    #[schema(example = "web")]
    pub guard_name: String,
    /// Source of the permission (direct or from role)
    #[schema(example = "role:Manager")]
    pub source: Option<String>,
}

/// User Organization Collection Response with pagination
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserOrganizationCollection {
    /// Array of user organization relationships
    pub data: Vec<UserOrganizationResource>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number
    #[schema(example = 1)]
    pub current_page: u32,
    /// Items per page
    #[schema(example = 15)]
    pub per_page: u32,
    /// Total number of items
    #[schema(example = 150)]
    pub total: u64,
    /// Total number of pages
    #[schema(example = 10)]
    pub last_page: u32,
    /// URL for first page
    #[schema(example = "https://api.example.com/user-organizations?page=1")]
    pub first_page_url: Option<String>,
    /// URL for last page
    #[schema(example = "https://api.example.com/user-organizations?page=10")]
    pub last_page_url: Option<String>,
    /// URL for next page
    #[schema(example = "https://api.example.com/user-organizations?page=2")]
    pub next_page_url: Option<String>,
    /// URL for previous page
    pub prev_page_url: Option<String>,
    /// From item number
    #[schema(example = 1)]
    pub from: Option<u32>,
    /// To item number
    #[schema(example = 15)]
    pub to: Option<u32>,
}

impl UserOrganizationCollection {
    /// Create a collection response with pagination
    pub fn new(
        data: Vec<UserOrganization>,
        current_page: u32,
        per_page: u32,
        total: u64,
        base_url: Option<&str>,
    ) -> Self {
        let resources = UserOrganizationResource::collection(data);
        let last_page = ((total as f64) / (per_page as f64)).ceil() as u32;

        let base_url = base_url.unwrap_or("/api/user-organizations");

        let first_page_url = Some(format!("{}?page=1", base_url));
        let last_page_url = Some(format!("{}?page={}", base_url, last_page));
        let next_page_url = if current_page < last_page {
            Some(format!("{}?page={}", base_url, current_page + 1))
        } else {
            None
        };
        let prev_page_url = if current_page > 1 {
            Some(format!("{}?page={}", base_url, current_page - 1))
        } else {
            None
        };

        let from = if total > 0 {
            Some((current_page - 1) * per_page + 1)
        } else {
            None
        };
        let to = if total > 0 {
            Some(std::cmp::min(current_page * per_page, total as u32))
        } else {
            None
        };

        Self {
            data: resources,
            meta: PaginationMeta {
                current_page,
                per_page,
                total,
                last_page,
                first_page_url,
                last_page_url,
                next_page_url,
                prev_page_url,
                from,
                to,
            },
        }
    }
}

/// User Organization Summary Resource for dashboard/overview
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserOrganizationSummaryResource {
    /// Total number of active relationships
    #[schema(example = 25)]
    pub total_active: u64,
    /// Total number of inactive relationships
    #[schema(example = 5)]
    pub total_inactive: u64,
    /// Breakdown by organization type
    pub by_organization_type: Vec<OrganizationTypeCount>,
    /// Recent transfers (last 30 days)
    #[schema(example = 3)]
    pub recent_transfers: u64,
    /// Relationships ending soon (next 30 days)
    #[schema(example = 2)]
    pub ending_soon: u64,
}

/// Organization type count for summary
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationTypeCount {
    /// Organization type
    #[schema(example = "department")]
    pub organization_type: String,
    /// Count of relationships
    #[schema(example = 15)]
    pub count: u64,
    /// Count of active relationships
    #[schema(example = 12)]
    pub active_count: u64,
}

/// User Organization Activity Resource for audit trail
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserOrganizationActivityResource {
    /// Activity ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User organization relationship ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub user_organization_id: String,
    /// Type of activity (created, updated, activated, deactivated, transferred, deleted)
    #[schema(example = "transferred")]
    pub activity_type: String,
    /// Activity description
    #[schema(example = "User transferred from Engineering to Marketing")]
    pub description: String,
    /// User who performed the activity
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub performed_by: String,
    /// Changes made (JSON object)
    pub changes: Option<serde_json::Value>,
    /// When the activity occurred
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Hierarchy visualization resource
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationHierarchyResource {
    /// Organization ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Organization name
    #[schema(example = "Engineering Department")]
    pub name: String,
    /// Organization type
    #[schema(example = "department")]
    pub organization_type: String,
    /// Parent organization ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<String>,
    /// Number of users in this organization
    #[schema(example = 25)]
    pub user_count: u64,
    /// Number of active users in this organization
    #[schema(example = 23)]
    pub active_user_count: u64,
    /// Child organizations
    pub children: Vec<OrganizationHierarchyResource>,
}
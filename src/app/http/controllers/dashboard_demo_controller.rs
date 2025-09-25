use axum::response::IntoResponse;
use serde_json::json;
use crate::app::http::responses::template_response::TemplateResponse;

pub struct DashboardDemoController;

impl DashboardDemoController {
    pub async fn index() -> impl IntoResponse {
        let data = json!({
            "title": "Dashboard",
            "current_route": "dashboard",
            "user": {
                "id": "1",
                "name": "John Doe",
                "first_name": "John",
                "email": "john@example.com",
                "avatar_url": "https://ui-avatars.com/api/?name=John+Doe&background=667eea&color=fff&size=128",
                "organization": {
                    "id": "1",
                    "name": "Acme Corp"
                },
                "can": {
                    "manage_users": true,
                    "manage_oauth": true,
                    "manage_system": true
                }
            },
            "breadcrumbs": [],
            "pending_users_count": 3,
            "unread_notifications_count": 7,
            "notifications_enabled": true,
            "websocket_enabled": false,
            "stats": [
                {
                    "label": "Total Users",
                    "value": "1,247",
                    "icon": "fas fa-users",
                    "color": {
                        "primary": "#667eea",
                        "secondary": "#764ba2"
                    },
                    "trend": {
                        "direction": "up",
                        "percentage": 12,
                        "period": "from last month"
                    }
                },
                {
                    "label": "Active Sessions",
                    "value": "89",
                    "icon": "fas fa-chart-line",
                    "color": {
                        "primary": "#28a745",
                        "secondary": "#20c997"
                    },
                    "trend": {
                        "direction": "up",
                        "percentage": 5,
                        "period": "from yesterday"
                    }
                },
                {
                    "label": "API Requests",
                    "value": "15.2K",
                    "icon": "fas fa-code",
                    "color": {
                        "primary": "#ffc107",
                        "secondary": "#fd7e14"
                    },
                    "trend": {
                        "direction": "down",
                        "percentage": 3,
                        "period": "from last week"
                    }
                },
                {
                    "label": "Revenue",
                    "value": "$12,847",
                    "icon": "fas fa-dollar-sign",
                    "color": {
                        "primary": "#e83e8c",
                        "secondary": "#dc3545"
                    },
                    "trend": {
                        "direction": "up",
                        "percentage": 18,
                        "period": "from last month"
                    }
                }
            ],
            "activity_chart": {
                "labels": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                "data": [120, 190, 300, 500, 200, 300, 450]
            },
            "user_distribution": {
                "labels": ["Admin", "Moderator", "User", "Guest"],
                "data": [15, 35, 180, 47]
            },
            "recent_activities": [
                {
                    "id": "1",
                    "description": "New user registration",
                    "details": "sarah.johnson@example.com joined the platform",
                    "icon": "fas fa-user-plus",
                    "type": "success",
                    "user": {
                        "name": "Sarah Johnson",
                        "avatar_url": "https://ui-avatars.com/api/?name=Sarah+Johnson&background=28a745&color=fff&size=48"
                    },
                    "time_ago": "2 minutes ago",
                    "timestamp": "2024-01-15T10:45:00Z",
                    "status": "Completed",
                    "status_color": "success"
                },
                {
                    "id": "2",
                    "description": "OAuth client created",
                    "details": "Mobile App v2.0 client credentials generated",
                    "icon": "fas fa-key",
                    "type": "info",
                    "user": {
                        "name": "John Doe",
                        "avatar_url": "https://ui-avatars.com/api/?name=John+Doe&background=667eea&color=fff&size=48"
                    },
                    "time_ago": "15 minutes ago",
                    "timestamp": "2024-01-15T10:32:00Z",
                    "status": "Active",
                    "status_color": "primary"
                },
                {
                    "id": "3",
                    "description": "System backup completed",
                    "details": "Daily backup finished successfully (2.3GB)",
                    "icon": "fas fa-database",
                    "type": "primary",
                    "user": {
                        "name": "System",
                        "avatar_url": "https://ui-avatars.com/api/?name=System&background=6c757d&color=fff&size=48"
                    },
                    "time_ago": "1 hour ago",
                    "timestamp": "2024-01-15T09:47:00Z",
                    "status": "Completed",
                    "status_color": "success"
                },
                {
                    "id": "4",
                    "description": "Failed login attempt",
                    "details": "Multiple failed attempts from IP 192.168.1.100",
                    "icon": "fas fa-shield-alt",
                    "type": "warning",
                    "user": {
                        "name": "Security System",
                        "avatar_url": "https://ui-avatars.com/api/?name=Security&background=dc3545&color=fff&size=48"
                    },
                    "time_ago": "2 hours ago",
                    "timestamp": "2024-01-15T08:30:00Z",
                    "status": "Blocked",
                    "status_color": "danger"
                }
            ],
            "recent_notifications": [
                {
                    "id": "1",
                    "title": "System Update Available",
                    "message": "Version 2.1.0 is ready for installation",
                    "icon": "fas fa-download",
                    "type": "info",
                    "time_ago": "10 minutes ago",
                    "read": false
                },
                {
                    "id": "2",
                    "title": "New Message",
                    "message": "You have a new message from the support team",
                    "icon": "fas fa-envelope",
                    "type": "primary",
                    "time_ago": "30 minutes ago",
                    "read": false
                },
                {
                    "id": "3",
                    "title": "Backup Completed",
                    "message": "Your daily backup was completed successfully",
                    "icon": "fas fa-check-circle",
                    "type": "success",
                    "time_ago": "1 hour ago",
                    "read": true
                }
            ],
            "system_status": [
                {
                    "name": "Database",
                    "description": "PostgreSQL 14.2",
                    "status_color": "success"
                },
                {
                    "name": "Redis Cache",
                    "description": "Redis 7.0.5",
                    "status_color": "success"
                },
                {
                    "name": "Email Service",
                    "description": "SMTP Connection",
                    "status_color": "warning"
                },
                {
                    "name": "File Storage",
                    "description": "85% capacity used",
                    "status_color": "success"
                }
            ],
            "quick_actions": [
                {
                    "title": "Add User",
                    "description": "Create new user account",
                    "icon": "fas fa-user-plus",
                    "url": "/dashboard/users/create",
                    "color": "primary"
                },
                {
                    "title": "View Reports",
                    "description": "Analytics & insights",
                    "icon": "fas fa-chart-bar",
                    "url": "/dashboard/reports",
                    "color": "success"
                },
                {
                    "title": "System Settings",
                    "description": "Configure application",
                    "icon": "fas fa-cog",
                    "url": "/dashboard/settings",
                    "color": "secondary"
                },
                {
                    "title": "API Documentation",
                    "description": "View API reference",
                    "icon": "fas fa-book",
                    "url": "/docs/swagger",
                    "color": "info"
                }
            ]
        });

        TemplateResponse::new("dashboard/index", &data).with_layout("layouts/dashboard")
    }

    pub async fn users() -> impl IntoResponse {
        let data = json!({
            "title": "User Management",
            "current_route": "users",
            "user": {
                "id": "1",
                "name": "John Doe",
                "email": "john@example.com",
                "avatar_url": "https://ui-avatars.com/api/?name=John+Doe&background=667eea&color=fff&size=128",
                "can": {
                    "manage_users": true,
                    "manage_oauth": true,
                    "manage_system": true
                }
            },
            "breadcrumbs": [
                {
                    "title": "Users",
                    "url": "/dashboard/users"
                }
            ],
            "unread_notifications_count": 5,
            "notifications_enabled": true
        });

        // Create users table template
        let users_table_data = json!({
            "title": "System Users",
            "icon": "fas fa-users",
            "table_id": "usersTable",
            "searchable": true,
            "selectable": true,
            "filters": [
                {
                    "name": "status",
                    "type": "select",
                    "placeholder": "Filter by status",
                    "width": "2",
                    "options": [
                        {"value": "active", "label": "Active"},
                        {"value": "inactive", "label": "Inactive"},
                        {"value": "suspended", "label": "Suspended"}
                    ]
                },
                {
                    "name": "role",
                    "type": "select",
                    "placeholder": "Filter by role",
                    "width": "2",
                    "options": [
                        {"value": "admin", "label": "Administrator"},
                        {"value": "user", "label": "User"},
                        {"value": "moderator", "label": "Moderator"}
                    ]
                },
                {
                    "name": "created_at",
                    "type": "date-range",
                    "placeholder": "Registration date",
                    "width": "4"
                },
                {
                    "name": "search",
                    "type": "text",
                    "placeholder": "Search users...",
                    "width": "4"
                }
            ],
            "actions": [
                {
                    "label": "Add User",
                    "icon": "fas fa-plus",
                    "url": "/dashboard/users/create",
                    "type": "primary"
                },
                {
                    "label": "Export",
                    "icon": "fas fa-download",
                    "url": "/dashboard/users/export",
                    "type": "outline-secondary"
                }
            ],
            "columns": [
                {
                    "label": "User",
                    "key": "name",
                    "type": "avatar",
                    "sortable": true,
                    "sort_key": "name",
                    "avatar_key": "avatar_url",
                    "name_key": "name"
                },
                {
                    "label": "Email",
                    "key": "email",
                    "sortable": true,
                    "sort_key": "email"
                },
                {
                    "label": "Role",
                    "key": "role",
                    "type": "badge",
                    "sortable": true,
                    "sort_key": "role"
                },
                {
                    "label": "Status",
                    "key": "status",
                    "type": "badge",
                    "sortable": true,
                    "sort_key": "status"
                },
                {
                    "label": "Last Login",
                    "key": "last_login",
                    "type": "date",
                    "sortable": true,
                    "sort_key": "last_login_at",
                    "format": "MMM DD, YYYY"
                },
                {
                    "label": "Joined",
                    "key": "created_at",
                    "type": "date",
                    "sortable": true,
                    "sort_key": "created_at",
                    "format": "MMM DD, YYYY"
                }
            ],
            "data": [
                {
                    "id": "1",
                    "name": "John Doe",
                    "email": "john@example.com",
                    "avatar_url": "https://ui-avatars.com/api/?name=John+Doe&background=667eea&color=fff&size=64",
                    "role": "Administrator",
                    "role_color": "danger",
                    "status": "Active",
                    "status_color": "success",
                    "last_login": "2024-01-15",
                    "created_at": "2023-01-15"
                },
                {
                    "id": "2",
                    "name": "Sarah Johnson",
                    "email": "sarah@example.com",
                    "avatar_url": "https://ui-avatars.com/api/?name=Sarah+Johnson&background=28a745&color=fff&size=64",
                    "role": "User",
                    "role_color": "primary",
                    "status": "Active",
                    "status_color": "success",
                    "last_login": "2024-01-14",
                    "created_at": "2023-03-22"
                },
                {
                    "id": "3",
                    "name": "Mike Wilson",
                    "email": "mike@example.com",
                    "avatar_url": "https://ui-avatars.com/api/?name=Mike+Wilson&background=ffc107&color=000&size=64",
                    "role": "Moderator",
                    "role_color": "warning",
                    "status": "Inactive",
                    "status_color": "secondary",
                    "last_login": "2024-01-10",
                    "created_at": "2023-06-15"
                }
            ],
            "has_actions": true,
            "row_actions": [
                {
                    "type": "dropdown",
                    "items": [
                        {
                            "label": "View Profile",
                            "icon": "fas fa-user",
                            "url": "/dashboard/users/{{id}}"
                        },
                        {
                            "label": "Edit",
                            "icon": "fas fa-edit",
                            "url": "/dashboard/users/{{id}}/edit"
                        },
                        {
                            "divider": true
                        },
                        {
                            "label": "Suspend",
                            "icon": "fas fa-ban",
                            "url": "/dashboard/users/{{id}}/suspend"
                        },
                        {
                            "label": "Delete",
                            "icon": "fas fa-trash",
                            "url": "/dashboard/users/{{id}}/delete"
                        }
                    ]
                }
            ],
            "bulk_actions": [
                {
                    "label": "Activate",
                    "action": "activate",
                    "icon": "fas fa-check",
                    "style": "success"
                },
                {
                    "label": "Suspend",
                    "action": "suspend",
                    "icon": "fas fa-ban",
                    "style": "warning",
                    "confirm": "Are you sure you want to suspend the selected users?"
                },
                {
                    "label": "Delete",
                    "action": "delete",
                    "icon": "fas fa-trash",
                    "style": "danger",
                    "confirm": "Are you sure you want to delete the selected users? This action cannot be undone."
                }
            ],
            "pagination": {
                "current_page": 1,
                "last_page": 5,
                "from": 1,
                "to": 20,
                "total": 87,
                "links": [
                    {"label": "1", "url": "?page=1", "active": true},
                    {"label": "2", "url": "?page=2", "active": false},
                    {"label": "3", "url": "?page=3", "active": false},
                    {"label": "...", "url": null, "disabled": true},
                    {"label": "5", "url": "?page=5", "active": false}
                ]
            }
        });

        let page_data = json!({
            "users_table": users_table_data
        });

        let combined_data = json!({
            "title": data["title"],
            "current_route": data["current_route"],
            "user": data["user"],
            "breadcrumbs": data["breadcrumbs"],
            "unread_notifications_count": data["unread_notifications_count"],
            "notifications_enabled": data["notifications_enabled"],
            "page_data": page_data
        });

        // For now, create a simple users listing page
        let template_content = r#"
<div class="page-header">
    <h1 class="page-title">User Management</h1>
    <p class="page-subtitle">Manage user accounts, roles, and permissions</p>
</div>

{{> components/tables/data-table page_data.users_table}}
"#;

        TemplateResponse::new("dashboard/users", &combined_data).with_layout("layouts/dashboard")
    }
}
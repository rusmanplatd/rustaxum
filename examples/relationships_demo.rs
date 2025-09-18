use rustaxum::query_builder::{
    Relationship, FilterOperator
};
use std::collections::HashMap;

/// Example demonstrating all Laravel relationship types in RustAxum Query Builder
fn main() {
    println!("=== Laravel Relationship Types in RustAxum Query Builder ===\n");

    // Basic relationships
    demo_basic_relationships();

    // Through relationships
    demo_through_relationships();

    // Polymorphic relationships
    demo_polymorphic_relationships();

    // Relationships with constraints
    demo_constrained_relationships();
}

fn demo_basic_relationships() {
    println!("1. Basic Relationships:");

    // HasOne: User has one Profile
    let user_profile = Relationship::has_one("user_id", "id", "profiles");
    println!("   HasOne: {:?}", user_profile);

    // HasMany: User has many Posts
    let user_posts = Relationship::has_many("user_id", "id", "posts");
    println!("   HasMany: {:?}", user_posts);

    // BelongsTo: Post belongs to User
    let post_user = Relationship::belongs_to("user_id", "id", "users");
    println!("   BelongsTo: {:?}", post_user);

    // BelongsToMany: User belongs to many Roles (with pivot table)
    let user_roles = Relationship::belongs_to_many(
        "id", "id", "roles", "user_roles", "user_id", "role_id"
    );
    println!("   BelongsToMany: {:?}\n", user_roles);
}

fn demo_through_relationships() {
    println!("2. Through Relationships:");

    // HasOneThrough: Country has one Profile through User
    let country_profile = Relationship::has_one_through(
        "id", "id", "profiles", "users", "country_id", "user_id", "id"
    );
    println!("   HasOneThrough: {:?}", country_profile);

    // HasManyThrough: Country has many Posts through Users
    let country_posts = Relationship::has_many_through(
        "id", "id", "posts", "users", "country_id", "user_id", "id"
    );
    println!("   HasManyThrough: {:?}\n", country_posts);
}

fn demo_polymorphic_relationships() {
    println!("3. Polymorphic Relationships:");

    // MorphTo: Comment morphs to (belongs to Post or Video)
    let comment_commentable = Relationship::morph_to("id", "commentable_type", "commentable_id");
    println!("   MorphTo: {:?}", comment_commentable);

    // MorphOne: Post morphs one Image
    let post_image = Relationship::morph_one(
        "id", "id", "images", "Post", "imageable_type", "imageable_id"
    );
    println!("   MorphOne: {:?}", post_image);

    // MorphMany: Post morphs many Comments
    let post_comments = Relationship::morph_many(
        "id", "id", "comments", "Post", "commentable_type", "commentable_id"
    );
    println!("   MorphMany: {:?}", post_comments);

    // MorphToMany: Post morphs to many Tags (with pivot table)
    let post_tags = Relationship::morph_to_many(
        "id", "id", "tags", "taggables", "post_id", "tag_id",
        "Post", "taggable_type", "taggable_id"
    );
    println!("   MorphToMany: {:?}\n", post_tags);
}

fn demo_constrained_relationships() {
    println!("4. Relationships with Constraints:");

    // User has many published Posts
    let published_posts = Relationship::has_many("user_id", "id", "posts")
        .with_constraint("status", FilterOperator::Eq, "published")
        .with_constraint("views", FilterOperator::Gt, "100");

    println!("   Constrained HasMany: {:?}", published_posts);

    // Generate SQL clause for constraints
    let (clause, params) = published_posts.build_constraint_clause();
    println!("   Generated SQL constraint: {}", clause);
    println!("   Parameters: {:?}", params);

    // User belongs to many active Roles
    let active_roles = Relationship::belongs_to_many(
        "id", "id", "roles", "user_roles", "user_id", "role_id"
    )
    .with_constraint("active", FilterOperator::Eq, "true")
    .with_constraint("name", FilterOperator::NotIn, "guest,banned");

    println!("   Constrained BelongsToMany: {:?}", active_roles);
    println!();
}

/// Example of how to define relationships in a model
struct User;

impl User {
    /// Define all user relationships
    fn relationships() -> HashMap<&'static str, Relationship> {
        let mut rels = HashMap::new();

        // Basic relationships
        rels.insert("profile", Relationship::has_one("user_id", "id", "profiles"));
        rels.insert("posts", Relationship::has_many("user_id", "id", "posts"));
        rels.insert("country", Relationship::belongs_to("country_id", "id", "countries"));
        rels.insert("roles", Relationship::belongs_to_many(
            "id", "id", "roles", "user_roles", "user_id", "role_id"
        ));

        // Constrained relationships
        rels.insert("published_posts",
            Relationship::has_many("user_id", "id", "posts")
                .with_constraint("status", FilterOperator::Eq, "published")
        );

        rels.insert("active_roles",
            Relationship::belongs_to_many(
                "id", "id", "roles", "user_roles", "user_id", "role_id"
            )
            .with_constraint("active", FilterOperator::Eq, "true")
        );

        // Through relationships
        rels.insert("country_posts", Relationship::has_many_through(
            "id", "id", "posts", "users", "country_id", "user_id", "id"
        ));

        // Polymorphic relationships
        rels.insert("images", Relationship::morph_many(
            "id", "id", "images", "User", "imageable_type", "imageable_id"
        ));

        rels.insert("tags", Relationship::morph_to_many(
            "id", "id", "tags", "taggables", "user_id", "tag_id",
            "User", "taggable_type", "taggable_id"
        ));

        rels
    }
}

/// Example usage with query parameters
fn example_query_usage() {
    println!("5. Example Query Usage:");
    println!("   GET /api/users?include=posts,roles,profile");
    println!("   GET /api/users?include=published_posts,active_roles");
    println!("   GET /api/posts?include=user,comments,tags");
    println!("   GET /api/posts?include=user.country,comments.user");
    println!();
}
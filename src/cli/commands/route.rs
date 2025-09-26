use anyhow::Result;
use std::fmt;
use std::fs;
use std::path::Path;
use colored::*;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub method: String,
    pub uri: String,
    pub name: Option<String>,
    pub controller: Option<String>,
    pub middleware: Vec<String>,
    pub route_type: RouteType,
}

#[derive(Debug, Clone)]
pub enum RouteType {
    Api,
    Web,
    OAuth,
}

impl fmt::Display for RouteType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RouteType::Api => write!(f, "API"),
            RouteType::Web => write!(f, "WEB"),
            RouteType::OAuth => write!(f, "OAUTH"),
        }
    }
}

pub async fn handle_route_list_command() -> Result<()> {
    let routes = discover_routes()?;
    display_routes(&routes);
    Ok(())
}

pub async fn handle_route_list_command_filtered(name_filter: Option<String>, method_filter: Option<String>, uri_filter: Option<String>) -> Result<()> {
    let mut routes = discover_routes()?;

    // Apply filters
    if let Some(name) = name_filter {
        routes.retain(|r| r.name.as_ref().map_or(false, |n| n.contains(&name)));
    }

    if let Some(method) = method_filter {
        routes.retain(|r| r.method.to_uppercase().contains(&method.to_uppercase()));
    }

    if let Some(uri) = uri_filter {
        routes.retain(|r| r.uri.contains(&uri));
    }

    display_routes(&routes);
    Ok(())
}

fn discover_routes() -> Result<Vec<RouteInfo>> {
    let mut routes = Vec::new();

    // Parse route files dynamically
    routes.extend(parse_route_file("src/routes/api.rs", RouteType::Api)?);
    routes.extend(parse_route_file("src/routes/web.rs", RouteType::Web)?);
    routes.extend(parse_route_file("src/routes/oauth.rs", RouteType::OAuth)?);

    // Sort routes by URI for better display
    routes.sort_by(|a, b| a.uri.cmp(&b.uri));

    Ok(routes)
}

fn parse_route_file(file_path: &str, route_type: RouteType) -> Result<Vec<RouteInfo>> {
    let mut routes = Vec::new();

    if !Path::new(file_path).exists() {
        return Ok(routes);
    }

    let content = fs::read_to_string(file_path)?;

    // Split content into separate router blocks
    let route_regex = Regex::new(r#"\.route\("([^"]+)",\s*(get|post|put|delete|patch)\(([^)]+)\)\)"#)?;
    let mut current_middleware = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Start of a new router block
        if line.starts_with("let ") && line.contains("Router::new()") {
            current_middleware.clear();

            // Look ahead to find if this block has middleware
            let mut j = i + 1;
            let mut routes_in_block = Vec::new();

            while j < lines.len() {
                let block_line = lines[j].trim();

                // Collect routes in this block
                if let Some(captures) = route_regex.captures(block_line) {
                    let uri = captures.get(1).unwrap().as_str().to_string();
                    let method = captures.get(2).unwrap().as_str().to_uppercase();
                    let controller_raw = captures.get(3).unwrap().as_str();

                    routes_in_block.push((uri, method, controller_raw.to_string()));
                }

                // Check for middleware
                if block_line.contains(".route_layer(") {
                    if block_line.contains("auth_guard") {
                        current_middleware.push("auth_guard".to_string());
                    } else if block_line.contains("guest_guard") {
                        current_middleware.push("guest_guard".to_string());
                    }
                }

                // End of block (semicolon at end)
                if block_line.ends_with(";") &&
                   (block_line.contains("route_layer") ||
                    (!block_line.contains(".route") && !block_line.contains("Router::new()"))) {
                    break;
                }

                j += 1;
            }

            // Add all routes from this block with the detected middleware
            for (uri, method, controller_raw) in routes_in_block {
                let name = generate_route_name(&uri, &method, &route_type);
                let route_info = RouteInfo {
                    method,
                    uri,
                    name: Some(name),
                    controller: Some(controller_raw),
                    middleware: current_middleware.clone(),
                    route_type: route_type.clone(),
                };
                routes.push(route_info);
            }

            i = j;
        }

        i += 1;
    }

    Ok(routes)
}

fn generate_route_name(uri: &str, method: &str, route_type: &RouteType) -> String {
    let prefix = match route_type {
        RouteType::Api => "api",
        RouteType::Web => "web",
        RouteType::OAuth => "oauth",
    };

    // Clean up URI for naming
    let clean_uri = uri
        .trim_start_matches("/api")
        .trim_start_matches("/oauth")
        .trim_start_matches("/")
        .replace("/", ".")
        .replace("{", "")
        .replace("}", "")
        .replace("-", "_");

    let action = match method {
        "GET" if clean_uri.is_empty() => "index",
        "GET" if clean_uri.ends_with("_id") || clean_uri.contains("_id.") => "show",
        "GET" => "index",
        "POST" => "store",
        "PUT" | "PATCH" => "update",
        "DELETE" => "destroy",
        _ => "unknown",
    };

    if clean_uri.is_empty() {
        format!("{}.{}", prefix, action)
    } else {
        format!("{}.{}.{}", prefix, clean_uri, action)
    }
}

fn display_routes(routes: &[RouteInfo]) {
    if routes.is_empty() {
        println!("{}", "No routes found matching the criteria.".yellow());
        return;
    }

    // Calculate column widths
    let method_width = routes.iter()
        .map(|r| r.method.len())
        .max()
        .unwrap_or(6)
        .max(6);

    let uri_width = routes.iter()
        .map(|r| r.uri.len())
        .max()
        .unwrap_or(3)
        .max(3);

    let name_width = routes.iter()
        .filter_map(|r| r.name.as_ref())
        .map(|n| n.len())
        .max()
        .unwrap_or(4)
        .max(4);

    let controller_width = routes.iter()
        .filter_map(|r| r.controller.as_ref())
        .map(|c| c.len())
        .max()
        .unwrap_or(10)
        .max(10);

    // Print header
    println!();
    println!("{}", format!("{:<method_width$} {:<uri_width$} {:<name_width$} {:<controller_width$} {:<10} {:<10}",
        "METHOD", "URI", "NAME", "CONTROLLER", "MIDDLEWARE", "TYPE",
        method_width = method_width,
        uri_width = uri_width,
        name_width = name_width,
        controller_width = controller_width
    ).bright_blue().bold());

    println!("{}", format!("{:-<width$}", "-", width = method_width + uri_width + name_width + controller_width + 32).bright_blue());

    // Print routes
    for route in routes {
        let method_color = match route.method.as_str() {
            "GET" => route.method.bright_blue(),
            "POST" => route.method.bright_green(),
            "PUT" => route.method.bright_yellow(),
            "DELETE" => route.method.bright_red(),
            "PATCH" => route.method.bright_magenta(),
            _ => route.method.normal(),
        };

        let type_color = match route.route_type {
            RouteType::Api => route.route_type.to_string().bright_cyan(),
            RouteType::Web => route.route_type.to_string().bright_green(),
            RouteType::OAuth => route.route_type.to_string().bright_yellow(),
        };

        let middleware_str = if route.middleware.is_empty() {
            "-".dimmed()
        } else {
            route.middleware.join(",").normal()
        };

        println!("{:<method_width$} {:<uri_width$} {:<name_width$} {:<controller_width$} {:<10} {:<10}",
            method_color,
            route.uri,
            route.name.as_deref().unwrap_or("-").dimmed(),
            route.controller.as_deref().unwrap_or("-").dimmed(),
            middleware_str,
            type_color,
            method_width = method_width,
            uri_width = uri_width,
            name_width = name_width,
            controller_width = controller_width
        );
    }

    println!();
    println!("{}", format!("Total routes: {}", routes.len()).bright_white().bold());

    // Statistics
    let api_count = routes.iter().filter(|r| matches!(r.route_type, RouteType::Api)).count();
    let web_count = routes.iter().filter(|r| matches!(r.route_type, RouteType::Web)).count();
    let oauth_count = routes.iter().filter(|r| matches!(r.route_type, RouteType::OAuth)).count();

    println!("{}", format!("API: {}, Web: {}, OAuth: {}", api_count, web_count, oauth_count).dimmed());
    println!();
}
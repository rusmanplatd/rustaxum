use anyhow::Result;
use tokio::time::{sleep, Duration};
use crate::app::broadcasting;

/// Handle broadcast test command
pub async fn handle_broadcast_test_command(channel: Option<String>, message: Option<String>) -> Result<()> {
    let channel = channel.unwrap_or_else(|| "general".to_string());
    let message = message.unwrap_or_else(|| "Test broadcast message".to_string());

    println!("🚀 Testing broadcast to channel: {}", channel);

    let data = serde_json::json!({
        "message": message,
        "timestamp": chrono::Utc::now(),
        "test": true
    });

    // Broadcast using the helper function
    if let Err(e) = broadcasting::helpers::broadcast_to_channel(&channel, "test", data).await {
        eprintln!("❌ Broadcast test failed: {}", e);
        return Err(e);
    }

    println!("✅ Broadcast test completed successfully");
    Ok(())
}

/// Handle websocket server command
pub async fn handle_websocket_serve_command(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(8080);

    println!("🌐 Starting WebSocket server on port {}", port);

    // Create WebSocket server
    if let Err(e) = broadcasting::websocket::create_websocket_server(port).await {
        eprintln!("❌ Failed to start WebSocket server: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Handle broadcast stats command
pub async fn handle_broadcast_stats_command() -> Result<()> {
    println!("📊 Broadcasting System Statistics");
    println!("================================");

    // Get WebSocket manager stats
    let ws_manager = broadcasting::websocket::websocket_manager().await;
    let active_channels = ws_manager.active_channels().await;

    if active_channels.is_empty() {
        println!("📡 WebSocket Channels: None active");
    } else {
        println!("📡 WebSocket Channels:");
        for channel in &active_channels {
            let connections = ws_manager.connection_count(channel).await;
            println!("  • {}: {} connections", channel, connections);
        }
    }

    // Get broadcast manager info
    let broadcast_manager = broadcasting::broadcast_manager().await;
    let _manager = broadcast_manager.read().await;

    println!();
    println!("🔧 Broadcast Configuration:");
    println!("  • Default Driver: Available");
    println!("  • Total Active Channels: {}", active_channels.len());

    if active_channels.is_empty() {
        println!();
        println!("💡 Tip: Start the WebSocket server with 'cargo run --bin artisan -- broadcast:websocket --port 8080'");
    }

    Ok(())
}

/// Handle broadcast ping command
pub async fn handle_broadcast_ping_command(channel: Option<String>, interval: Option<u64>) -> Result<()> {
    let channel = channel.unwrap_or_else(|| "general".to_string());
    let interval_secs = interval.unwrap_or(5);

    println!("🏓 Starting broadcast ping to channel '{}' every {} seconds", channel, interval_secs);
    println!("Press Ctrl+C to stop");

    let mut counter = 1;

    loop {
        let data = serde_json::json!({
            "ping": counter,
            "timestamp": chrono::Utc::now(),
            "message": format!("Ping #{}", counter)
        });

        match broadcasting::helpers::broadcast_to_channel(&channel, "ping", data).await {
            Ok(_) => println!("📡 Ping #{} sent to channel '{}'", counter, channel),
            Err(e) => eprintln!("❌ Failed to send ping #{}: {}", counter, e),
        }

        counter += 1;
        sleep(Duration::from_secs(interval_secs)).await;
    }
}

/// Handle broadcast channels command
pub async fn handle_broadcast_channels_command() -> Result<()> {
    println!("📻 Active Broadcast Channels");
    println!("===========================");

    let ws_manager = broadcasting::websocket::websocket_manager().await;
    let active_channels = ws_manager.active_channels().await;

    if active_channels.is_empty() {
        println!("No active channels found.");
        println!();
        println!("💡 Channels are created when clients connect to them.");
        println!("   Try connecting to WebSocket endpoint: ws://localhost:3000/ws?channel=test");
        return Ok(());
    }

    for channel in active_channels {
        let connections = ws_manager.connection_count(&channel).await;
        let status = if connections > 0 { "🟢 Active" } else { "🟡 Idle" };

        println!("📡 {} ({} connections) - {}", channel, connections, status);
    }

    Ok(())
}

/// Handle broadcast to user command
pub async fn handle_broadcast_to_user_command(user_id: String, title: String, message: String, action_url: Option<String>) -> Result<()> {
    println!("👤 Broadcasting to user: {}", user_id);

    if let Err(e) = broadcasting::helpers::broadcast_notification(&user_id, &title, &message, action_url.as_deref()).await {
        eprintln!("❌ Failed to broadcast to user: {}", e);
        return Err(e);
    }

    println!("✅ Notification sent to user {} successfully", user_id);
    Ok(())
}

/// Handle system alert command
pub async fn handle_system_alert_command(level: String, message: String, action_required: Option<bool>) -> Result<()> {
    let action_required = action_required.unwrap_or(false);

    println!("🚨 Broadcasting system alert ({})", level.to_uppercase());

    if let Err(e) = broadcasting::helpers::broadcast_system_alert(&level, &message, action_required).await {
        eprintln!("❌ Failed to broadcast system alert: {}", e);
        return Err(e);
    }

    println!("✅ System alert broadcast successfully");
    Ok(())
}

/// Handle broadcast monitor command
pub async fn handle_broadcast_monitor_command(duration: Option<u64>) -> Result<()> {
    let duration_secs = duration.unwrap_or(30);

    println!("👀 Monitoring broadcast activity for {} seconds", duration_secs);
    println!("Watching for new connections and broadcasts...");
    println!("Press Ctrl+C to stop early");

    let ws_manager = broadcasting::websocket::websocket_manager().await;
    let start_time = std::time::Instant::now();

    while start_time.elapsed().as_secs() < duration_secs {
        let active_channels = ws_manager.active_channels().await;
        let total_connections: usize = {
            let mut total = 0;
            for channel in &active_channels {
                total += ws_manager.connection_count(channel).await;
            }
            total
        };

        print!("\r📊 Channels: {} | Connections: {} | Time: {}s",
               active_channels.len(),
               total_connections,
               start_time.elapsed().as_secs());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        sleep(Duration::from_secs(1)).await;
    }

    println!();
    println!("✅ Monitoring completed");
    Ok(())
}
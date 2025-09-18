use crate::cli::{MakeCommands, generators};
use anyhow::Result;

pub async fn handle_make_command(command: MakeCommands) -> Result<()> {
    match command {
        MakeCommands::Controller { name, resource } => {
            generators::controller::generate_controller(&name, resource).await
        },
        MakeCommands::Model { name, migration } => {
            generators::model::generate_model(&name, migration).await
        },
        MakeCommands::Service { name } => {
            generators::service::generate_service(&name).await
        },
        MakeCommands::Middleware { name } => {
            generators::middleware::generate_middleware(&name).await
        },
        MakeCommands::Migration { name } => {
            generators::migration::generate_migration(&name).await
        },
        MakeCommands::Request { name } => {
            generators::request::generate_request(&name).await
        },
        MakeCommands::Seeder { name } => {
            generators::seeder::generate_seeder(&name).await
        },
    }
}
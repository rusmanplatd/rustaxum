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
        MakeCommands::Resource { name, collection } => {
            generators::resource::generate_resource(&name, collection).await
        },
        MakeCommands::Mail { name, markdown } => {
            generators::mail::generate_mail(&name, markdown).await
        },
        MakeCommands::Notification { name, markdown } => {
            generators::notification::generate_notification(&name, markdown).await
        },
        MakeCommands::Job { name, sync } => {
            generators::job::generate_job(&name, sync).await
        },
        MakeCommands::Event { name } => {
            generators::event::generate_event(&name).await
        },
        MakeCommands::Listener { name, event, queued } => {
            generators::listener::generate_listener(&name, event, queued).await
        },
        MakeCommands::Policy { name, model } => {
            generators::policy::generate_policy(&name, model).await
        },
        MakeCommands::Rule { name } => {
            generators::rule::generate_rule(&name).await
        },
        MakeCommands::Test { name, unit } => {
            generators::test::generate_test(&name, unit).await
        },
    }
}
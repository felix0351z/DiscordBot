use std::default::Default;
use std::sync::Arc;

use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::events::Events;
use lavalink_rs::node::NodeBuilder;
use lavalink_rs::prelude::NodeDistributionStrategy;
use poise::{Framework, FrameworkError, FrameworkOptions, PrefixFrameworkOptions};
use serenity::all::GatewayIntents;
use serenity::Client;
use songbird::SerenityInit;

// General command
mod commands;

// Event handler
mod events;

// Error handler
mod error;

// All music related code
mod music;

// Config manager
mod config;

// Custom user data passed to all command functions
pub struct Data {
    lavalink: Arc<LavalinkClient>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[tokio::main]
async fn main() {
    // Start the logger and load  settings
    env_logger::init();
    let settings = config::load_settings();

    // Provide intents, which are needed for this bot
    let intents =
        GatewayIntents::all();

    //Initialise the poise framework for command management
    let options = FrameworkOptions {
        commands: vec![
            commands::hello(), commands::ping(), commands::help(),
            music::play::play(), music::skip::skip(), music::stop::stop(),
            music::info::info(), music::queue::queue(), music::clear::clear(),
            music::leave::leave(), music::lavalink::lavalink(),
        ],
        prefix_options: PrefixFrameworkOptions {
            prefix: Some(settings.application.prefix),
            mention_as_prefix: true,
            ..Default::default()
        }, //Global error handler for all errors which occur
        on_error: |framework_err: FrameworkError<'_, Data, Error>| {
            Box::pin(error::error_handler(framework_err))
        },
        event_handler: |ctx, event, framework, _data| {
            Box::pin(events::event_handler(ctx, event, framework))
        },
        ..Default::default()
    };

    let poise_framework = Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // Register the commands of the bot at the discord server
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Load lavalink
                let events = Events {
                    ready: Some(music::ready_event),
                    ..Default::default()
                };
                let node = NodeBuilder {
                        hostname: format!("{}:{}", settings.lavalink.hostname, settings.lavalink.port).to_string(),
                        password: settings.lavalink.password,
                        is_ssl: settings.lavalink.is_ssl,
                        events: Events::default(),
                        user_id: ctx.cache.current_user().id.into(),
                        session_id: None
                };
                let client = LavalinkClient::new(events, vec![node], NodeDistributionStrategy::round_robin()).await;

                Ok(Data {
                    lavalink: Arc::new(client)
                })
            })
        })
        .options(options)
        .build();




    // Create the serenity client and start the server
    let mut client = Client::builder(settings.application.discord_token, intents)
        .register_songbird()
        .framework(poise_framework)
        .await
        .unwrap_or_else(|err| {
            panic!("Error occurred on client creation: {}", err)
        });

    // Start the client
    if let Err(error) = client.start().await {
        println!("Error while client runtime: {error:?}")
    }
}






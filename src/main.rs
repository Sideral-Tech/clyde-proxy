#![warn(clippy::str_to_string)]

mod commands;

use poise::serenity_prelude::{self as serenity, ChannelId, UserId};
use std::env::var;
use std::sync::Arc;
use tokio::sync::Semaphore;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const CLYDE_ID: u64 = 1081004946872352958;

#[derive(Default)]
struct Handler {
    options: poise::FrameworkOptions<Data, Error>,
    command_names: Vec<String>,
    data: Data,
    bot_id: UserId,
    shard_manager: std::sync::Mutex<Option<Arc<tokio::sync::Mutex<serenity::ShardManager>>>>,
}

#[derive(Default)]
pub struct Data {
    config: tokio::sync::Mutex<Config>,
    locking: Locking,
}

#[derive(Default)]
pub struct Config {
    proxy_config: Option<ProxyConfiguration>,
}

pub struct Locking {
    shared_state: tokio::sync::Mutex<Option<Arc<State>>>,
    semaphore: Semaphore,
}

pub struct State {
    pub last_message: serenity::Message,
}

#[derive(Default)]
pub struct ProxyConfiguration {
    to_channel_id: ChannelId,   // The channel ID to proxy to.
    from_channel_id: ChannelId, // The channel ID to proxy from.
    enabled: bool,              // Whether the proxy is enabled.
}

impl Default for Locking {
    fn default() -> Self {
        Self {
            shared_state: Default::default(),
            semaphore: Semaphore::new(1),
        }
    }
}

// Custom handler to dispatch poise events.
impl Handler {
    pub fn new(options: poise::FrameworkOptions<Data, Error>, bot_id: u64) -> Self {
        let command_names = options
            .commands
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>();

        Self {
            options,
            command_names,
            bot_id: UserId(bot_id),
            ..Default::default()
        }
    }

    async fn dispatch_poise_event(&self, ctx: &serenity::Context, event: &poise::Event<'_>) {
        let framework_data = poise::FrameworkContext {
            bot_id: self.bot_id,
            options: &self.options,
            user_data: &self.data,
            shard_manager: &(*self.shard_manager.lock().unwrap()).clone().unwrap(),
        };

        poise::dispatch_event(framework_data, ctx, event).await;
    }
}

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, ctx: serenity::Context, new_message: serenity::Message) {
        // Respond back.
        if new_message.author.id == CLYDE_ID {
            if let Some(proxy_config) = &self.data.config.lock().await.proxy_config {
                // TODO: If the message is in a thread check for the channel id in which the thread was.
                if proxy_config.to_channel_id == new_message.channel_id {
                    // Reply to the message and release the lock.
                    self.data.locking.semaphore.add_permits(1);

                    let state = self.data.locking.shared_state.lock().await.clone().unwrap();

                    let _ = proxy_config
                        .from_channel_id
                        .send_message(&ctx.http, |b| {
                            b.reference_message(&state.last_message);
                            b.content(&new_message.content)
                        })
                        .await;
                }
            }
        } else {
            let proxy = if let Some(referenced_message) = &new_message.referenced_message {
                referenced_message.author.id == self.bot_id
            } else {
                new_message.author.id != self.bot_id && new_message.mentions_user_id(self.bot_id)
            };

            if proxy {
                let message_to_proxy = &new_message
                    .content
                    .replace(&format!("<@{}>", self.bot_id), "");

                let trimmed_message_to_proxy = message_to_proxy.trim();
                if !self
                    .command_names
                    .iter()
                    .any(|name| trimmed_message_to_proxy.starts_with(name))
                {
                    // Lock the event bus to prevent other messages from being sent before Clyde replied to the sent message.
                    self.data
                        .locking
                        .semaphore
                        .acquire()
                        .await
                        .unwrap()
                        .forget();

                    if let Some(ref mut proxy_config) =
                        &mut self.data.config.lock().await.proxy_config
                    {
                        proxy_config.from_channel_id = new_message.channel_id;

                        let _ = proxy_config
                            .to_channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "<@{}> Hello, my name is {}. {}",
                                    CLYDE_ID, new_message.author, message_to_proxy
                                ),
                            )
                            .await;

                        self.data.locking.shared_state.lock().await.replace(
                            State {
                                last_message: new_message.clone(),
                            }
                            .into(),
                        );
                    }
                }
            }
        }

        self.dispatch_poise_event(&ctx, &poise::Event::Message { new_message })
            .await;
    }

    async fn interaction_create(&self, ctx: serenity::Context, interaction: serenity::Interaction) {
        self.dispatch_poise_event(&ctx, &poise::Event::InteractionCreate { interaction })
            .await;
    }
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx } => {
            let response = &format!("Error in command `{}`: {:?}", ctx.command().name, error);

            ctx.say(response).await.ok();

            println!("{}", response);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv::dotenv().expect("Failed to read .env file");

    let options = poise::FrameworkOptions {
        commands: vec![commands::help(), commands::toggle(), commands::proxy()],
        prefix_options: poise::PrefixFrameworkOptions {
            mention_as_prefix: true,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}", ctx.command().qualified_name);
            })
        },
        owners: vec![UserId(
            var("OWNER_ID")
                .expect("Missing `OWNER_ID` environment variable")
                .parse::<u64>()
                .expect("Failed to parse `OWNER_ID` environment variable"),
        )]
        .into_iter()
        .collect(),
        skip_checks_for_owners: false,
        ..Default::default()
    };

    let handler = std::sync::Arc::new(Handler::new(
        options,
        var("SELF_BOT_USER_ID")
            .expect("Missing `DISCORD_TOKEN` environment variable")
            .parse()
            .unwrap(),
    ));

    let mut client = serenity::Client::builder(
        var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` environment variable"),
        serenity::GatewayIntents::non_privileged()
            | serenity::GatewayIntents::MESSAGE_CONTENT
            | serenity::GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler_arc(handler.clone())
    .await?;

    *handler.shard_manager.lock().unwrap() = Some(client.shard_manager.clone());
    client.start().await?;

    Ok(())
}

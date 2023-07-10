use poise::serenity_prelude::CacheHttp;

use crate::{Context, Error, ProxyConfiguration, CLYDE_ID};

/// Show this help menu.
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Proxy Clyde from one Discord channel to another.",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
pub async fn toggle(ctx: Context<'_>) -> Result<(), Error> {
    let mut config = ctx.data().config.lock().await;

    let Some(ref mut proxy_config) = &mut config.proxy_config
    else { return Err("No proxy configured".into()); };

    proxy_config.enabled = !proxy_config.enabled;

    if proxy_config.enabled {
        ctx.say("Proxy enabled.").await?;
    } else {
        ctx.say("Proxy disabled.").await?;
    }

    Ok(())
}

/// Proxy Clyde from another Discord channel.
#[poise::command(prefix_command, owners_only)]
pub async fn proxy(
    ctx: Context<'_>,
    #[description = "The channel ID to proxy Clyde from"] channel_id: String,
) -> Result<(), Error> {
    let Ok(channel) = ctx.http().get_channel(channel_id.parse()?).await else {
        return Err("Can not find channel ID".into());
    };

    let config = ProxyConfiguration {
        from_channel_id: ctx.channel_id(),
        to_channel_id: channel.id(),
        enabled: true,
    };

    ctx.data().config.lock().await.proxy_config = Some(config);

    ctx.say(format!(
        "Proxying <@{}> from channel <#{}>.",
        CLYDE_ID, channel_id
    ))
    .await?;

    Ok(())
}

/// Send a message to the proxied server.
#[poise::command(prefix_command, aliases("m"))]
pub async fn message(
    ctx: Context<'_>,
    #[description = "The message to send"] message: String,
) -> Result<(), Error> {
    let mut config = ctx.data().config.lock().await;

    let Some(ref mut proxy_config) = &mut config.proxy_config
        else { return Err("No proxy configured".into()); };

    if !proxy_config.enabled {
        return Err("Proxy is not enabled".into());
    }

    // Remember the current channel ID so we can send a message back to it.
    proxy_config.from_channel_id = ctx.channel_id();

    proxy_config.to_channel_id.say(
        ctx.http(),
        format!(
            "<@{}> Hello, my name is {}. {}",
            CLYDE_ID,
            ctx.author().name,
            message
        ),
    ).await?;

    Ok(())
}

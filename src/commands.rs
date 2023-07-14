
use crate::{Context, Error, ProxyConfiguration, CLYDE_ID};
use poise::serenity_prelude::{CacheHttp};

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

// Toggle the proxy on or off.
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

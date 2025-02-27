use crate::{Context, Error};
use crate::music::MusicCommandError::NoTrackIsPlaying;
use crate::music::PlayerStoppedExtension;

/// Stop the playback
#[poise::command(prefix_command, slash_command, guild_only, category = "Music")]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {

    let guild_id = ctx.guild_id().unwrap();
    let lavalink = ctx.data().lavalink.clone();

    let Some(player) = lavalink.get_player_context(guild_id) else {
        return Err(NoTrackIsPlaying.into())
    };

    let now_playing = player.get_player().await?.track.is_some();
    if !now_playing {
        return Err(NoTrackIsPlaying.into());
    }

    player.stop_now().await?;
    player.mark_stop()?;
    ctx.say("Wiedegabe beendet!").await?;


    return Ok(())
}

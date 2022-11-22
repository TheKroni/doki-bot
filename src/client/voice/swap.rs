use serenity::{
    builder::EditInteractionResponse,
    client::Context,
    model::prelude::interaction::application_command::{
        ApplicationCommandInteraction, CommandDataOptionValue,
    },
};
use songbird::tracks::TrackQueue;
use thiserror::Error;

use super::{
    helper_funcs::{
        get_call_lock, is_bot_in_another_voice_channel, voice_channel_not_same_response,
    },
    MyAuxMetadata,
};

pub trait Swapable {
    fn swap(&self, first_track_idx: usize, second_track_idx: usize) -> Result<(), SwapableError>;
}

#[derive(Debug, Error)]
pub enum SwapableError {
    #[error("Requested song that wasn't in the queue")]
    IndexOutOfBounds,
    #[error("Can't swap any songs if the queue is empty")]
    NothingIsPlaying,
    #[error("Can't swap the song that's currently being played")]
    CannotSwapCurrentSong,
    #[error("Can't swap a song with itself")]
    CannotSwapSameSong,
}

impl Swapable for TrackQueue {
    fn swap(&self, first_track_idx: usize, second_track_idx: usize) -> Result<(), SwapableError> {
        self.modify_queue(|q| {
            if q.len() < first_track_idx
                || q.len() < second_track_idx
                || first_track_idx < 1
                || second_track_idx < 1
            {
                return Err(SwapableError::IndexOutOfBounds);
            }

            if q.is_empty() {
                return Err(SwapableError::NothingIsPlaying);
            }

            if first_track_idx == 1 || second_track_idx == 1 {
                return Err(SwapableError::CannotSwapCurrentSong);
            }

            if first_track_idx == second_track_idx {
                return Err(SwapableError::CannotSwapSameSong);
            }

            let first_track_idx = first_track_idx - 1;
            let second_track_idx = second_track_idx - 1;

            q.swap(first_track_idx, second_track_idx);

            Ok(())
        })
    }
}

pub async fn swap(ctx: &Context, command: &ApplicationCommandInteraction) {
    let guild_id = command.guild_id.expect("Couldn't get guild ID");
    let guild = guild_id.to_guild_cached(&ctx.cache).map(|g| g.to_owned());

    command.defer(&ctx.http).await.unwrap();

    if let Some(guild) = guild {
        if is_bot_in_another_voice_channel(ctx, &guild, command.user.id) {
            voice_channel_not_same_response(command, ctx).await;
            return;
        }
    }

    let call_lock = match get_call_lock(ctx, guild_id, command).await {
        Some(value) => value,
        None => return,
    };

    let call = call_lock.lock().await;

    let queue = call.queue();

    let (first_track_idx, second_track_idx) = if let Some(v) = get_track_numbers(command) {
        v
    } else {
        invalid_number_response(command, ctx).await;
        return;
    };

    let (first_track_idx, second_track_idx) =
        if let Ok(value) = parse_track_numbers(first_track_idx, second_track_idx) {
            value
        } else {
            invalid_number_response(command, ctx).await;
            return;
        };

    let first_track = get_track_from_queue(queue, first_track_idx).await;

    let second_track = get_track_from_queue(queue, second_track_idx).await;

    match queue.swap(first_track_idx, second_track_idx) {
        Ok(_) => {
            swapping_success_response(command, ctx, first_track_idx, first_track, second_track_idx, second_track).await;
        }
        Err(e) => swapping_error_response(e, command, ctx).await,
    }
}

async fn swapping_success_response(command: &ApplicationCommandInteraction, ctx: &Context, first_track_idx: usize, first_track: MyAuxMetadata, second_track_idx: usize, second_track: MyAuxMetadata) {
    command
        .edit_original_interaction_response(
            &ctx.http,
            EditInteractionResponse::new().content(format!(
                "
Swapped tracks \n
{}. {}\n
and\n
{}. {}
",
                first_track_idx,
                first_track
                    .0
                    .title
                    .unwrap_or_else(|| "NO TITLE".to_string()),
                second_track_idx,
                second_track
                    .0
                    .title
                    .unwrap_or_else(|| "NO TITLE".to_string())
            )),
        )
        .await
        .expect("Error creating interaction response");
}

async fn get_track_from_queue(queue: &TrackQueue, track_number: usize) -> MyAuxMetadata {
    let second_track = queue
        .current_queue()
        .get(track_number - 1)
        .unwrap()
        .typemap()
        .read()
        .await
        .get::<MyAuxMetadata>()
        .unwrap()
        .read()
        .unwrap()
        .clone();
    second_track
}

async fn invalid_number_response(command: &ApplicationCommandInteraction, ctx: &Context) {
    command
        .edit_original_interaction_response(
            &ctx.http,
            EditInteractionResponse::new().content("Invalid number!"),
        )
        .await
        .expect("Error creating interaction response");
}

fn parse_track_numbers(
    first_track_idx: i64,
    second_track_idx: i64,
) -> anyhow::Result<(usize, usize)> {
    let first_track_idx = first_track_idx.try_into()?;
    let second_track_idx = second_track_idx.try_into()?;

    Ok((first_track_idx, second_track_idx))
}

async fn swapping_error_response(
    e: SwapableError,
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) {
    match e {
        SwapableError::IndexOutOfBounds => {
            command
                .edit_original_interaction_response(
                    &ctx.http,
                    EditInteractionResponse::new().content("That track isn't in the queue!"),
                )
                .await
                .expect("Error creating interaction response");
        }
        SwapableError::NothingIsPlaying => {
            command
                .edit_original_interaction_response(
                    &ctx.http,
                    EditInteractionResponse::new().content("Nothing is playing!"),
                )
                .await
                .expect("Error creating interaction response");
        }
        SwapableError::CannotSwapCurrentSong => {
            command
                .edit_original_interaction_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content("Can't swap the song that's currently playing!"),
                )
                .await
                .expect("Error creating interaction response");
        }
        SwapableError::CannotSwapSameSong => {
            command
                .edit_original_interaction_response(
                    &ctx.http,
                    EditInteractionResponse::new().content("Can't swap the same song!"),
                )
                .await
                .expect("Error creating interaction response");
        }
    }
}

fn get_track_numbers(command: &ApplicationCommandInteraction) -> Option<(i64, i64)> {
    let first_track_idx = match command.data.options.get(0).unwrap().value {
        CommandDataOptionValue::Integer(i) => i,
        _ => return None,
    };

    let second_track_idx = match command.data.options.get(1).unwrap().value {
        CommandDataOptionValue::Integer(i) => i,
        _ => return None,
    };

    Some((first_track_idx, second_track_idx))
}

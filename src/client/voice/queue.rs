use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    utils::Colour,
};

///get the queue
pub async fn queue(ctx: &Context, command: &ApplicationCommandInteraction) {
    let cache = &ctx.cache;
    let guild_id = command.guild_id;
    if let Some(_guild) = cache.guild(guild_id.unwrap()).await {
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if let Some(handler_lock) = manager.get(guild_id.unwrap()) {
            let handler = handler_lock.lock().await;
            let queue = handler.queue();

            if queue.is_empty() {
                command
                    .create_interaction_response(&ctx.http, |r| {
                        r.interaction_response_data(|d| d.content("The queue is empty!"))
                    })
                    .await
                    .expect("Error creating interaction response");
                return;
            }
            //embed
            command
                .create_interaction_response(&ctx.http, |m| {
                    //embed
                    let i: usize;
                    if queue.len() < 10 {
                        i = queue.len();
                    } else {
                        i = 10;
                    }
                    //color
                    let colour = Colour::from_rgb(149, 8, 2);
                    let total_queue_time = queue
                        .current_queue()
                        .iter()
                        .map(|f| f.metadata().duration.unwrap())
                        .reduce(|a, f| a.checked_add(f).unwrap())
                        .unwrap_or_default();

                    let minutes = total_queue_time.as_secs() / 60;
                    let seconds = total_queue_time.as_secs() - minutes * 60;
                    let duration = format!("{}:{:02}", minutes, seconds);

                    m.interaction_response_data(|d| {
                        d.create_embed(|e| {
                            e.title("queue")
                                .title("Current Queue:")
                                .description(format!(
                                    "Current size: {} | Total queue length: {}",
                                    queue.len(),
                                    duration
                                ))
                                .color(colour);
                            for i in 0..i {
                                let song =
                                    &queue.current_queue().get(i).unwrap().metadata().clone();
                                let channel = &song.channel.as_ref().unwrap();
                                let title = &song.title.as_ref().unwrap();
                                //duration
                                let time = &song.duration.as_ref().unwrap();
                                let minutes = time.as_secs() / 60;
                                let seconds = time.as_secs() - minutes * 60;
                                let duration = format!("{}:{:02}", minutes, seconds);
                                let arg1 = format!("{}. {} | {}", i + 1, title, channel);
                                e.field(arg1, duration, false);
                            }
                            e
                        })
                    })
                })
                .await
                .expect("Error creating interaction response");
        } else {
            command
                .create_interaction_response(&ctx.http, |r| {
                    r.interaction_response_data(|d| {
                        d.content("You must be in a voice channel to use that command!")
                    })
                })
                .await
                .expect("Error creating interaction response");
        }
    }
}
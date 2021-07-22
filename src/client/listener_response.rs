use crate::*;
use regex::Regex;
use serenity::{
    client::Context,
    model::{
        id::UserId,
        interactions::{
            ApplicationCommandInteractionData, ApplicationCommandInteractionDataOptionValue,
            ApplicationCommandOptionType,
        },
        prelude::User,
    },
};

pub async fn list_listeners(ctx: &Context) -> String {
    let listener_response_lock = get_listener_response_lock(&ctx.data).await;
    let listener_response = listener_response_lock.read().await;

    let mut message = String::new();

    for (listener, _) in listener_response.iter() {
        message += &format!("{}, ", listener);
    }
    message.pop();
    message.pop();

    message
}

pub async fn remove_listener_command(
    ctx: &Context,
    data: &ApplicationCommandInteractionData,
) -> String {
    let listener = data
        .options
        .get(0)
        .expect("expected listener")
        .resolved
        .as_ref()
        .unwrap();
    let listener_response_lock = get_listener_response_lock(&ctx.data).await;

    let mut listener_response = listener_response_lock.write().await;

    if let ApplicationCommandInteractionDataOptionValue::String(listener) = listener {
        if listener_response.contains_key(listener) {
            listener_response.remove(listener);
            save_listener_response_to_file(&listener_response);
            return "Successfully removed the listener".to_string();
        } 
            return "That listener doesn't exist".to_string();
        
    }

    "Something went wrong".to_string()
}

pub async fn set_listener_command(
    ctx: &Context,
    data: &ApplicationCommandInteractionData,
) -> String {
    let listener = data
        .options
        .get(0)
        .expect("expected listener")
        .resolved
        .as_ref()
        .unwrap();
    let response = data
        .options
        .get(1)
        .expect("expected response")
        .resolved
        .as_ref()
        .unwrap();

    if let ApplicationCommandInteractionDataOptionValue::String(listener) = listener {
        if let ApplicationCommandInteractionDataOptionValue::String(response) = response {
            let user_regex = Regex::new(r"<@!?(\d+)>").unwrap();
            let role_regex = Regex::new(r"<@&(\d+)>").unwrap();
            if user_regex.is_match(response)
                || user_regex.is_match(listener)
                || role_regex.is_match(response)
                || role_regex.is_match(listener)
                || response.contains("@everyone")
                || response.contains("@here")
            {
                return "can't add a mention".to_string();
            }

            let listener_response_lock = get_listener_response_lock(&ctx.data).await;

            let mut listener_response = listener_response_lock.write().await;
            listener_response.insert(
                listener.to_lowercase().trim().to_string(),
                response.trim().to_string(),
            );
            save_listener_response_to_file(&listener_response);
            return "Set listener".to_string();
        }
    }
    "Couldn't set listener".to_string()
}

pub async fn blacklist_user_from_listener(ctx: &Context, user: &User) -> String {
    let listener_blacklisted_users_lock = get_listener_blacklisted_users_lock(&ctx.data).await;

    let mut users_blacklisted_from_listener = listener_blacklisted_users_lock.write().await;

    if users_blacklisted_from_listener.contains(&user.id.0) {
        users_blacklisted_from_listener.remove(&user.id.0);
        save_user_listener_blacklist_to_file(&users_blacklisted_from_listener);
        "Removed user from the blacklist".to_string()
    } else {
        users_blacklisted_from_listener.insert(user.id.0);
        save_user_listener_blacklist_to_file(&users_blacklisted_from_listener);
        "Added user to the blacklist".to_string()
    }
}

///Checks for all the listened words in the message
///
///If a listened word is found it returns the response
pub async fn check_for_listened_words(
    ctx: &Context,
    words_in_message: &[String],
    user_id: UserId,
) -> Option<String> {
    let listener_response_lock = get_listener_response_lock(&ctx.data).await;
    let listener_response = listener_response_lock.read().await;
    let listener_blacklisted_users_lock = get_listener_blacklisted_users_lock(&ctx.data).await;
    let listener_blacklisted_users = listener_blacklisted_users_lock.read().await;
    for (listener, response) in listener_response.iter() {
        if words_in_message.contains(&listener) && !listener_blacklisted_users.contains(&user_id.0)
        {
            return Some(response.to_string());
        }
    }
    None
}

pub fn create_listener_commands(
    commands: &mut serenity::builder::CreateApplicationCommands,
) -> &mut serenity::builder::CreateApplicationCommands {
    commands.create_application_command(|command| {
            command.name("setlistener").description(
                "Start a listener for a word or list of words and a response whenever someone says that word",
            )
            .create_option(|option|{
                option.name("listenedword").description("What word to listen for").kind(ApplicationCommandOptionType::String).required(true)
            })
            .create_option(|option|{
                option.name("response").description("What the response should be when the listened word is said")
                .kind(ApplicationCommandOptionType::String)
                .required(true)
            })
        })
        .create_application_command(|command| {
            command.name("removelistener").description("Remove a listener from a word").create_option(|option|{
                option.name("listenedword").description("The word to remove").kind(ApplicationCommandOptionType::String).required(true)
            })
        })
        .create_application_command(|command|{
            command.name("listeners").description("List all of the listeners")
        })
        .create_application_command(|command|{
            command.name("blacklistlistener").description("The bot won't respond to your messages if you trip off a listener")
        })
}
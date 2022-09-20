use log::{error, info};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::Guild;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::*;

use shuttle_service::error::CustomError;
use shuttle_service::SecretStore;
use sqlx::PgPool;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Ping(_) => todo!(),
            Interaction::ApplicationCommand(cmd) => {info!("Recieved command: {}", cmd.data.name)},
            Interaction::MessageComponent(_) => todo!(),
            Interaction::Autocomplete(_) => todo!(),
            Interaction::ModalSubmit(_) => todo!(),
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        info!("The guild {} ({}) has added the bot", guild.name, guild.id);
        
        let set_command = guild.set_application_commands(ctx.http, |commands| { // Creating Commands
            commands
                .create_application_command(|command| { // Main Ballot Command
                    command
                        .name("ballot")
                        .description("Create, manage, and edit ballots.")
                        .kind(CommandType::ChatInput)
                        .create_option(|option| {
                            option
                                .name("new")
                                .kind(CommandOptionType::SubCommand)
                                .description("Create new ballot")
                                .create_sub_option(|option| {
                                    option
                                        .name("question")
                                        .kind(CommandOptionType::String)
                                        .min_length(3)
                                        .description("The question the poll should ask")
                                        .required(true)
                                })
                                .create_sub_option(|option| {
                                    option
                                        .name("type")
                                        .kind(CommandOptionType::String)
                                        .description("Kind of ballot")
                                        .add_string_choice("Preference Voting", "pf")
                                        .add_string_choice("Ranked Choice Voting", "rc")
                                        .add_string_choice("Score Voting", "sr")
                                        .add_string_choice("First Past The Post", "fp")
                                })
                        })
                })
        }).await;
        match set_command {
            Ok(_) => {},
            Err(x) => {error!("Error setting the commands: {}", x)}
        }
    }
}

#[shuttle_service::main]
async fn serenity(#[shared::Postgres] pool: PgPool) -> shuttle_service::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml` from the shared Postgres database
    let token = pool
        .get_secret("DISCORD_TOKEN")
        .await
        .map_err(CustomError::new)?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_INTEGRATIONS | GatewayIntents::GUILD_MESSAGE_REACTIONS | GatewayIntents::GUILDS;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client)
}
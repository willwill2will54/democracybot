use log::{error, info};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::Guild;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Ping(_) => {},
            Interaction::ApplicationCommand(cmd) => self.handle_command(ctx, cmd).await,
            Interaction::MessageComponent(_) => {},
            Interaction::Autocomplete(_) => {},
            Interaction::ModalSubmit(_) => {},
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn guild_create(&self, ctx: Context, guild: Guild) {
        info!("The guild {} ({}) has added the bot", guild.name, guild.id);
        
        let set_command = guild.set_application_commands(ctx.http, |commands| { // Creating Commands
            commands
                .create_application_command(|command| { // Main Ballot Command
                    command
                        .name("ballot")
                        .description("Create, manage, and edit ballots")
                        .kind(CommandType::ChatInput)
                        .create_option(|option| { // New Ballot
                            option
                                .name("new")
                                .kind(CommandOptionType::SubCommand)
                                .description("Create new ballot")
                                .create_sub_option(|option| { // Ballot Question
                                    option
                                        .name("question")
                                        .kind(CommandOptionType::String)
                                        .min_length(3)
                                        .description("The question the poll should ask")
                                        .required(true)
                                })
                                .create_sub_option(|option| { // Ballot type
                                    option
                                        .name("type")
                                        .kind(CommandOptionType::String)
                                        .description("Kind of ballot")
                                        .add_string_choice("Preference Voting", "pf")
                                        .add_string_choice("Ranked Choice Voting", "rc")
                                        .add_string_choice("Score Voting", "sr")
                                        .add_string_choice("First Past The Post", "fp")
                                        .required(true)
                                })
                                .create_sub_option(|option| { // Ballot option number
                                    option
                                        .name("options")
                                        .kind(CommandOptionType::Integer)
                                        .description("Number of options")
                                        .min_int_value(2)
                                        .required(true)
                                })
                        })
                })
                .create_application_command(|command| { // Fate Fudge command
                    command
                        .name("rollfate")
                        .description("Roll fudge dice on a fate skill check")
                        .kind(CommandType::ChatInput)
                        .create_option(|option| {
                            option
                                .name("base")
                                .kind(CommandOptionType::Integer)
                                .description("The fate skill level to offset")
                                .add_int_choice("Terrible", -2)
                                .add_int_choice("Poor", -1)
                                .add_int_choice("Mediocre", -0)
                                .add_int_choice("Average", 1)
                                .add_int_choice("Fair", 2)
                                .add_int_choice("Good", 3)
                                .add_int_choice("Great", 4)
                                .add_int_choice("Superb", 5)
                                .add_int_choice("Fantastic", 6)
                                .add_int_choice("Epic", 7)
                                .add_int_choice("Legendary", 8)
                                .min_int_value(-2)
                                .max_int_value(8)
                        })
                        .create_option(|option| {
                            option
                                .name("fudge")
                                .description("The number of fudge dice to roll.")
                                .kind(CommandOptionType::Integer)
                                .min_int_value(0)
                                .max_int_value(5)  
                        })
                })
        }).await;
        match set_command {
            Ok(_) => {},
            Err(x) => {error!("Error setting the commands: {}", x)}
        }
    }
}

impl Bot {
    async fn handle_command(&self, ctx: Context, cmd: ApplicationCommandInteraction) {
        if let Err(err) = match cmd.data.name.as_str() {
            "ballot" => self.ballot(ctx, cmd).await,
            "rollfate"  => self.rollfate(ctx, cmd).await,
            x => {
                error!("Unknown command: {}", x);
                cmd.create_interaction_response(ctx.http, |response| {
                    response
                        .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|data| {
                            data
                                .content("That command is not known!")
                        })
                }).await
            }
        } {
            error!("Error creating command response: {}", err)
        }
    }

    async fn ballot(&self, ctx: Context, cmd: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
        let number_of_options = cmd.data.options.get(0).unwrap().options.get(2).unwrap().value.as_ref().unwrap().as_u64().unwrap();

        cmd.create_interaction_response(ctx.http, |response| {
            response
                .kind(serenity::model::prelude::interaction::InteractionResponseType::Modal)
                .interaction_response_data(|data| {
                    data
                        .components(|mut components| {
                            for i in 0..number_of_options {
                                components = components
                                    .create_action_row(|actions| {
                                        actions.create_input_text(|text| {
                                            text
                                                .custom_id(format!("option_{}", i))
                                                .label(format!("Option {}", i + 1))
                                                .min_length(2)
                                                .max_length(100)
                                                .required(true)
                                                .placeholder("Some Option")
                                                .style(serenity::model::prelude::component::InputTextStyle::Short)
                                        })
                                    })
                            }
                            components
                        })
                        .custom_id("modal")
                        .title(cmd.data.options.get(0).unwrap().options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap())
                })
        }).await
    }

    async fn rollfate(&self, _ctx: Context, _cmd: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
        todo!()
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
use ballot::VotingTypeEmoji;
use log::{error, info};
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        guild::Guild,
        application::interaction::{
            Interaction,
            application_command::ApplicationCommandInteraction
        },
        prelude::{
            ReactionType,
            interaction::modal::ModalSubmitInteraction,
            command::{
                CommandType,
                CommandOptionType
            }
        }
    },
    prelude::*
};

use shuttle_service::error::CustomError;
use shuttle_service::SecretStore;
use sqlx::PgPool;

use crate::dice::fate_number_to_rank;

mod ballot;
mod dice;

struct EmojiStore;

impl EmojiStore {
    pub(crate) fn get_emoji(&self, emoji: VotingTypeEmoji) -> ReactionType {
        match emoji {
            FPTP => ReactionType::from('✉'),
            PREF => ReactionType::from('⚙'),
            RANK => ReactionType::from('🥇'),
            SCORE => ReactionType::from('🅱'),
            _ => ReactionType::from('❓')
        }
    }

    fn new() -> Self {
        Self
    }
}

struct Bot {
    emojis: EmojiStore
}

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
            Interaction::ModalSubmit(modal) => {},
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
                        .create_option(|mut option| {
                            option = option
                                .name("base")
                                .kind(CommandOptionType::Integer)
                                .description("The fate skill level to offset")
                                .min_int_value(-2)
                                .max_int_value(8);
                            
                            for i in -2 ..= 8 {
                                option = option.add_int_choice(fate_number_to_rank(i as i64), i);
                            }
                                
                            option
                                
                        })
                        .create_option(|option| {
                            option
                                .name("dice")
                                .description("The number of fudge dice to roll.")
                                .kind(CommandOptionType::Integer)
                                .min_int_value(0)
                                .max_int_value(6)  
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

    async fn handle_modal(&self, ctx: Context, modal: ModalSubmitInteraction) {
        match modal.data.custom_id {
            _ => {
                
            }
        }
    }

    fn new() -> Self {
        Self {emojis: EmojiStore::new()}
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
        .event_handler(Bot::new())
        .await
        .expect("Err creating client");

    Ok(client)
}
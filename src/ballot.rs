use serenity::{prelude::Context, model::prelude::interaction::application_command::ApplicationCommandInteraction};

use crate::Bot;

pub(super) enum VotingTypeEmoji {
    FPTP, PREF, RANK, SCORE
}
struct ballot;


impl Bot {
    pub(crate) async fn ballot(&self, ctx: Context, cmd: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
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
                            components.create_action_row(|actions| {
                                actions
                                    .create_select_menu(|menu| {
                                        menu
                                            .custom_id("vote-type")
                                            .placeholder("Select Vote Type")
                                            .options(|options| {
                                                options
                                                    .create_option(|option| {
                                                        option
                                                            .label("First Past The Post")
                                                            .value("fp")
                                                            .description("It sucks but people know it.")
                                                            .emoji(self.emojis.get_emoji(VotingTypeEmoji::FPTP))
                                                    })
                                                    .create_option(|option| {
                                                        option
                                                            .label("Preference Voting")
                                                            .value("pf")
                                                            .description("Which ones are ok by you? Simple Enough!")
                                                            .emoji(self.emojis.get_emoji(VotingTypeEmoji::FPTP))
                                                    })
                                                    .create_option(|option| {
                                                        option
                                                            .label("Score Voting")
                                                            .value("sr")
                                                            .description("How MUCH do you want that, exactly?")
                                                            .emoji(self.emojis.get_emoji(VotingTypeEmoji::FPTP))
                                                    })
                                                    .create_option(|option| {
                                                        option
                                                            .label("Ranked Choice Voting")
                                                            .value("rc")
                                                            .description("Rank them. Familiar to anyone who has read buzzfeed.")
                                                            .emoji(self.emojis.get_emoji(VotingTypeEmoji::FPTP))
                                                    })
                                            })
                                    })
                            })
                        })
                        .custom_id("modal")
                        .title(cmd.data.options.get(0).unwrap().options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap())
                })
        }).await
    }
}
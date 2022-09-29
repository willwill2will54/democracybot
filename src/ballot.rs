use serenity::{
    prelude::Context,
    model::prelude::interaction::application_command::ApplicationCommandInteraction
};

use crate::Bot;

pub(super) enum VotingTypeEmoji {
    FPTP, PREF, RANK, SCORE
}
struct ballot;

impl ballot {

}

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
                            components
                        })
                        .custom_id("modal")
                        .title(cmd.data.options.get(0).unwrap().options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap())
                })
        }).await
    }
}
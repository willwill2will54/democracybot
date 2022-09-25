use serenity::{prelude::Context, model::prelude::interaction::application_command::ApplicationCommandInteraction};

use crate::Bot;

impl Bot {
    pub(super) async fn rollfate(&self, _ctx: Context, _cmd: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
        todo!()
    }
}
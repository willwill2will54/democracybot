use std::{i32, fmt::format};
use log::error;

use rand::{RngCore, SeedableRng, rngs::StdRng};
use serenity::{
    prelude::Context,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction,
        InteractionResponseType
    }
};

use crate::Bot;


pub(super) fn fate_number_to_rank(num: i64) -> String {
    match num {
        -2 => "Terrible".to_string(),
        -1 => "Poor".to_string(),
        0  => "Mediocre".to_string(),
        1  => "Average".to_string(),
        2  => "Fair".to_string(),
        3  => "Good".to_string(),
        4  => "Great".to_string(),
        5  => "Superb".to_string(),
        6  => "Fantastic".to_string(),
        7  => "Epic".to_string(),
        8  => "Legendary".to_string(),
        x  => format!("{}", x),
    }
}

enum RollVec {
    RollVec(Vec<Roll>)
}

impl From<RollVec> for String {
    fn from(vec: RollVec) -> Self {
        let RollVec::RollVec(rolls) = vec;
        rolls.iter()
            .map(|roll| {
                match roll {
                    Roll::PLUS => "🔼",
                    Roll::MINUS => "🔽",
                    Roll::NEUTRAL => "⏺️",
                }
            })
            .fold("".to_string(), |acc, next| {
                format!("{} {}", acc, next)
            }) 
    }
}

enum Roll {
    PLUS, MINUS, NEUTRAL
}

impl Bot {
    #[doc(hidden)]
    // The documentation has already begun!
    pub(super) async fn rollfate(&self, ctx: Context, cmd: ApplicationCommandInteraction) -> Result<(), serenity::Error> {
        
        let mut number_of_dice = 4;
        let mut base: Option<i64> = None;

        for option in &cmd.data.options {
            match option.name.as_str() {
                "base" => {
                    base = Some(option.value.as_ref().unwrap().as_i64().unwrap());
                }
                "dice" => {
                    number_of_dice = option.value.as_ref().unwrap().as_u64().unwrap();
                }
                err => {
                    error!("Unrecognised option: {}", err)
                }
            }
        }
        
        let mut rng: StdRng = SeedableRng::from_entropy();        
        
        let mut offset = 0;
        let mut dice_sequence: Vec<Roll> = vec![];

        for _ in 0..number_of_dice {
            let mut number: u32;
            loop {
                number = rng.next_u32() % 4;
                match number {
                    1 => {
                        offset += 1;
                        dice_sequence.push(Roll::PLUS);
                    }
                    2 => {
                        offset -= 1;
                        dice_sequence.push(Roll::MINUS);
                    }
                    3 => {
                        dice_sequence.push(Roll::NEUTRAL);
                    }
                    _ => continue,
                }
                break;
            }
        }

        let rollstring: String = RollVec::RollVec(dice_sequence).into();

        let message = match base {
            None => {
                format!("You rolled {} dice and they came up as {}, giving a total of {}.", number_of_dice, rollstring, offset)
            }
            Some(base_value) => {
                format!(
                    "You rolled {} dice on a base of {} ({}) and they came up as {}, giving a total of {} ({}).",
                    number_of_dice,
                    fate_number_to_rank(base_value),
                    base_value,
                    rollstring,
                    fate_number_to_rank(base_value + offset),
                    base_value + offset
                )
            }
        };

        cmd.create_interaction_response(ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data
                        .content(message)
                })
        }).await
    }
}
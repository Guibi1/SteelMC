use std::sync::Arc;

use crate::command::commands::{CommandExecutor, CommandHandler, literal};
use crate::command::context::CommandContext;
use crate::command::error::CommandError;
use crate::server::Server;

/// The handler for the "weather" command.
pub(self) fn init_weather_command_handler() -> CommandHandler<WeatherCommandExecutor> {
    CommandHandler::new(
        &["weather"],
        "Changes the weather in the current dimension.",
        "minecraft:command.weather",
    )
    .then(literal("rain").executes(WeatherCommandExecutor::Rain))
    .then(literal("thunder").executes(WeatherCommandExecutor::Thunder))
    .then(literal("clear").executes(WeatherCommandExecutor::Clear))
}

enum WeatherCommandExecutor {
    Clear,
    Rain,
    Thunder,
}

impl CommandExecutor<()> for WeatherCommandExecutor {
    fn execute(
        &self,
        _args: (),
        _server: &Arc<Server>,
        _context: &mut CommandContext,
    ) -> Result<(), CommandError> {
        match self {
            WeatherCommandExecutor::Clear => {
                todo!()
            }
            WeatherCommandExecutor::Rain => {
                todo!()
            }
            WeatherCommandExecutor::Thunder => {
                todo!()
            }
        }
        Ok(())
    }
}

use crate::command::arguments::CommandArgument;
use crate::command::context::CommandContext;

pub struct RotationArgument;

impl CommandArgument for RotationArgument {
    type Output = (f32, f32);

    fn parse<'a>(
        &self,
        arg: &'a [&'a str],
        _context: &mut CommandContext,
    ) -> Option<(&'a [&'a str], Self::Output)> {
        let mut yaw = arg.get(0)?.parse::<f32>().ok()?;
        let mut pitch = arg.get(1)?.parse::<f32>().ok()?;

        yaw %= 360.0;
        if yaw >= 180.0 {
            yaw -= 360.0;
        }
        pitch %= 360.0;
        if pitch >= 180.0 {
            pitch -= 360.0;
        }

        Some((&arg[2..], (yaw, pitch)))
    }
}

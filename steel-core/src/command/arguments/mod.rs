//! This module contains types and utilities for parsing command arguments.
pub(super) mod literal;
pub(super) mod rotation;
pub(super) mod vector2;
pub(super) mod vector3;

use crate::command::context::CommandContext;

pub(super) trait CommandArgument {
    type Output;

    fn parse<'a>(
        &self,
        arg: &'a [&'a str],
        context: &mut CommandContext,
    ) -> Option<(&'a [&'a str], Self::Output)>;
}

// TODO: https://minecraft.wiki/w/Argument_types

pub(self) struct Helper;

impl Helper {
    pub fn parse_relative_coordinate<const IS_Y: bool>(
        s: &str,
        origin: Option<f64>,
    ) -> Option<f64> {
        if let Some(s) = s.strip_prefix('~') {
            let origin = origin?;
            let offset = if s.is_empty() { 0.0 } else { s.parse().ok()? };
            Some(origin + offset)
        } else {
            let mut v = s.parse().ok()?;

            // set position to block center if no decimal place is given
            if !IS_Y && !s.contains('.') {
                v += 0.5;
            }

            Some(v)
        }
    }
}

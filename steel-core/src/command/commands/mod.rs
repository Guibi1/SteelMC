//! This module contains the actual command implementations.
// pub mod gamemode;
// pub mod teleport;
pub mod weather;

use std::sync::Arc;

use crate::command::arguments::CommandArgument;
use crate::command::arguments::literal::LiteralArgument;
use crate::command::context::CommandContext;
use crate::command::error::CommandError;
use crate::server::Server;

/// Trait for command handler objects, allowing type-erased storage.
pub trait CommandHandlerDyn: Send + Sync {
    /// Returns the names and aliases of this command.
    fn names(&self) -> &'static [&'static str];
    /// Returns the description of this command.
    fn description(&self) -> &'static str;
    /// Returns the permission required to execute this command.
    fn permission(&self) -> &'static str;
    /// Try to run the command with the given unparsed arguments and context.
    fn execute(
        &self,
        command_args: &[&str],
        server: Arc<Server>,
        context: &mut CommandContext,
    ) -> Result<(), CommandError>;
}

/// The struct that holds command handler data and executor.
pub(super) struct CommandHandler<E> {
    /// The name and aliases of this command.
    pub names: &'static [&'static str],
    /// A description of this command.
    pub description: &'static str,
    /// The permission required to execute this command.    
    pub permission: &'static str,
    /// The command parser chain for this command.
    executor: Option<E>,
}

impl<E> CommandHandler<E>
where
    E: CommandExecutor<()>,
{
    /// Creates a new command handler.
    pub fn new(
        names: &'static [&'static str],
        description: &'static str,
        permission: &'static str,
    ) -> Self {
        CommandHandler {
            names,
            description,
            permission,
            executor: None,
        }
    }

    pub fn then(mut self, executor: E) -> Self {
        self.executor = Some(executor);
        self
    }

    pub fn executes(mut self, executor: E) -> Self {
        self.executor = Some(executor);
        self
    }
}

impl<E> CommandHandlerDyn for CommandHandler<E>
where
    E: for<'a> CommandParserExecutor<'a, ()> + Send + Sync,
{
    fn names(&self) -> &'static [&'static str] {
        self.names
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn permission(&self) -> &'static str {
        self.permission
    }

    fn execute(
        &self,
        command_args: &[&str],
        server: Arc<Server>,
        context: &mut CommandContext,
    ) -> Result<(), CommandError> {
        // TODO: Allow commands without args
        let Some(executor) = &self.executor else {
            unimplemented!(
                "Command {} has no executor defined. Please call `then()` or `executes()` on your CommandHandler.",
                self.names[0]
            );
        };

        let Some(result) = executor.execute(command_args, (), &server, context) else {
            return Err(CommandError::CommandFailed(
                format!("Invalid Syntax.").into(),
            ));
        };

        result
    }
}

/// A node in the command parser chain.
pub struct CommandParserArgNode<A, E> {
    /// The parser to use for this argument.
    argument: Box<dyn CommandArgument<Output = A>>,
    /// The executor to run if this node matches the command input.
    executor: E,
}

impl<'a, A, E, P> CommandParserExecutor<'a, P> for CommandParserArgNode<A, E>
where
    E: CommandParserExecutor<'a, (A, P)>,
{
    fn execute(
        &self,
        args: &'a [&'a str],
        parsed: P,
        server: &Arc<Server>,
        context: &mut CommandContext,
    ) -> Option<Result<(), CommandError>> {
        let Some((args, arg)) = self.argument.parse(args, context) else {
            return None;
        };

        self.executor.execute(args, (arg, parsed), &server, context)
    }
}

/// A leaf in the command parser chain.
pub struct CommandParserLeaf<E> {
    /// The function to run.
    executor: E,
}

impl<'a, E, P> CommandParserExecutor<'a, P> for CommandParserLeaf<E>
where
    E: Fn(&'a [&'a str], P, &Arc<Server>, &mut CommandContext) -> Result<(), CommandError>,
{
    fn execute(
        &self,
        args: &'a [&'a str],
        parsed: P,
        server: &Arc<Server>,
        context: &mut CommandContext,
    ) -> Option<Result<(), CommandError>> {
        if args.is_empty() {
            Some((self.executor)(args, parsed, &server, context))
        } else {
            None
        }
    }
}

pub(super) trait CommandParserExecutor<'a, P> {
    fn execute(
        &self,
        args: &'a [&'a str],
        parsed: P,
        server: &Arc<Server>,
        context: &mut CommandContext,
    ) -> Option<Result<(), CommandError>>;
}

pub(super) trait CommandExecutor<P> {
    fn execute(
        &self,
        parsed: P,
        server: &Arc<Server>,
        context: &mut CommandContext,
    ) -> Result<(), CommandError>;
}

struct LiteralBuilder {
    literal: LiteralArgument,
}

pub fn literal(expected: &'static str) -> LiteralBuilder {
    LiteralBuilder {
        literal: LiteralArgument { expected },
    }
}

impl<'a> LiteralBuilder {
    pub fn executes<E, P>(self, executor: E) -> CommandParserArgNode<(), E>
    where
        E: CommandExecutor<P>,
    {
        CommandParserArgNode {
            argument: Box::new(self.literal),
            executor,
        }
    }
}

struct ArgumentBuilder<A> {
    argument: Box<dyn CommandArgument<Output = A>>,
}

pub fn argument<A>(argument: impl CommandArgument<Output = A> + 'static) -> ArgumentBuilder<A> {
    ArgumentBuilder {
        argument: Box::new(argument),
    }
}

impl<'a, A> ArgumentBuilder<A> {
    pub fn executes<E, P>(self, executor: E) -> CommandParserArgNode<A, E>
    where
        E: CommandParserExecutor<'a, (A, P)>,
    {
        CommandParserArgNode {
            argument: self.argument,
            executor,
        }
    }
}

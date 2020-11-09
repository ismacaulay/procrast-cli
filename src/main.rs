mod auth;
mod cmd;
mod command;
mod config;
mod context;
mod input;
mod log;
mod models;
mod network;
mod output;
mod sqlite;
mod sync;
mod utils;

// TODO: look into the built package
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use crate::{
    command::{Command, CommandParams, Flag, FlagDescription},
    context::Context,
};
use std::env;

struct Cli {
    name: &'static str,
    description: &'static str,
    commands: Vec<Command>,
}

impl Cli {
    fn run(&self, ctx: &mut Context, args: &[String]) {
        if args.len() == 0 {
            self.print_help_and_exit(1);
        }

        let cmd = args[0].as_str();
        match cmd {
            "version" | "--version" => {
                println!("procrast-cli version {}", VERSION);
                std::process::exit(0);
            }
            "help" | "--help" | "-h" => self.print_help_and_exit(0),
            _ => {
                if let Some(command) = self
                    .commands
                    .iter()
                    .find(|c| c.name == cmd || c.aliases.iter().find(|a| **a == cmd) != None)
                {
                    command.run(ctx, &args[1..]);
                } else {
                    self.print_help_and_exit(1);
                }
            }
        };
    }

    fn print_help_and_exit(&self, code: i32) {
        let mut command_str = String::with_capacity(20);
        for c in self.commands.iter() {
            let s = format!("  {:<8}    {}\n", c.name, c.description);
            command_str.push_str(&s);
        }

        println!(
            "

Usage: {} COMMAND

{}

Commands:
{}",
            self.name, self.description, command_str
        );

        std::process::exit(code);
    }
}

fn main() {
    let app = Cli {
        name: "procrast",
        description: "A cli for managing your procrastination",
        commands: vec![
            Command {
                name: "login",
                aliases: vec![],
                description: "Login to the cloud",
                params: CommandParams::None,
                action: auth::login,
                flags: vec![],
                subcommands: vec![],
            },
            Command {
                name: "use",
                aliases: vec!["u"],
                description: "Set the default list",
                params: CommandParams::Single("LIST"),
                action: cmd::use_list,
                flags: vec![],
                subcommands: vec![],
            },
            Command {
                name: "sync",
                aliases: vec![],
                description: "Sync with the remote server",
                params: CommandParams::None,
                action: sync::run,
                flags: vec![Flag::Switch(FlagDescription {
                    name: "all",
                    short: "",
                    description: "sync all",
                })],
                subcommands: vec![],
            },
            Command {
                name: "list",
                aliases: vec!["l"],
                description: "Manage lists",
                params: CommandParams::None,
                action: cmd::list,
                flags: vec![],
                subcommands: vec![
                    Command {
                        name: "create",
                        aliases: vec!["c"],
                        description: "create a new list",
                        params: CommandParams::None,
                        action: cmd::list::create,
                        subcommands: vec![],
                        flags: vec![
                            Flag::Flag(FlagDescription {
                                name: "title",
                                short: "t",
                                description: "the list title",
                            }),
                            Flag::Flag(FlagDescription {
                                name: "desc",
                                short: "d",
                                description: "the list description",
                            }),
                        ],
                    },
                    Command {
                        name: "show",
                        aliases: vec!["s"],
                        description: "Show a list",
                        params: CommandParams::Single("LIST"),
                        action: cmd::list::show,
                        subcommands: vec![],
                        flags: vec![],
                    },
                    Command {
                        name: "edit",
                        aliases: vec!["e"],
                        description: "Edit an existing list",
                        params: CommandParams::Single("LIST"),
                        action: cmd::list::edit,
                        subcommands: vec![],
                        flags: vec![
                            Flag::Flag(FlagDescription {
                                name: "title",
                                short: "t",
                                description: "the list title",
                            }),
                            Flag::Flag(FlagDescription {
                                name: "desc",
                                short: "d",
                                description: "the list description",
                            }),
                        ],
                    },
                    Command {
                        name: "delete",
                        aliases: vec!["d"],
                        description: "Delete one or more lists",
                        params: CommandParams::Multi("LIST"),
                        action: cmd::list::delete,
                        subcommands: vec![],
                        flags: vec![],
                    },
                ],
            },
            Command {
                name: "item",
                aliases: vec!["i"],
                description: "Manage items",
                params: CommandParams::Single("ITEM"),
                action: cmd::item,
                flags: vec![
                    Flag::Flag(FlagDescription {
                        name: "list",
                        short: "l",
                        description: "The list to get the item from",
                    }),
                    Flag::Switch(FlagDescription {
                        name: "complete",
                        short: "c",
                        description: "Mark the item as complete",
                    }),
                    Flag::Switch(FlagDescription {
                        name: "incomplete",
                        short: "i",
                        description: "Mark the item as incomplete",
                    }),
                    Flag::Switch(FlagDescription {
                        name: "all",
                        short: "a",
                        description: "Show all items",
                    }),
                ],
                subcommands: vec![
                    Command {
                        name: "add",
                        aliases: vec!["a"],
                        description: "Add a new item to the list",
                        params: CommandParams::None,
                        action: cmd::item::add,
                        subcommands: vec![],
                        flags: vec![
                            Flag::Flag(FlagDescription {
                                name: "list",
                                short: "l",
                                description: "the list to add the item too",
                            }),
                            Flag::Flag(FlagDescription {
                                name: "title",
                                short: "t",
                                description: "the item title",
                            }),
                            Flag::Flag(FlagDescription {
                                name: "desc",
                                short: "d",
                                description: "the item description",
                            }),
                        ],
                    },
                    Command {
                        name: "show",
                        aliases: vec!["s"],
                        description: "Show an item",
                        params: CommandParams::Single("ITEM"),
                        action: cmd::item::show,
                        subcommands: vec![],
                        flags: vec![],
                    },
                    Command {
                        name: "edit",
                        aliases: vec!["e"],
                        description: "Edit an item in the list",
                        params: CommandParams::Single("ITEM"),
                        action: cmd::item::edit,
                        subcommands: vec![],
                        flags: vec![
                            Flag::Flag(FlagDescription {
                                name: "list",
                                short: "l",
                                description: "the list the item is in",
                            }),
                            Flag::Flag(FlagDescription {
                                name: "title",
                                short: "t",
                                description: "the item title",
                            }),
                            Flag::Flag(FlagDescription {
                                name: "desc",
                                short: "d",
                                description: "the item description",
                            }),
                        ],
                    },
                    Command {
                        name: "delete",
                        aliases: vec!["d"],
                        description: "Delete one or more items",
                        params: CommandParams::Multi("ITEM"),
                        action: cmd::item::delete,
                        subcommands: vec![],
                        flags: vec![Flag::Flag(FlagDescription {
                            name: "list",
                            short: "l",
                            description: "the list the item is in",
                        })],
                    },
                ],
            },
            cmd::notes::command(),
        ],
    };

    let mut ctx = Context::new();
    let args: Vec<String> = env::args().collect();
    app.run(&mut ctx, &args[1..]);
}

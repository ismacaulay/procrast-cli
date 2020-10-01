mod auth;
mod cmd;
mod config;
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

use std::collections::HashMap;
use std::env;

enum CommandParams {
    None,
    Single(&'static str),
    Multi(&'static str),
}

#[derive(Copy, Clone)]
struct FlagDescription {
    name: &'static str,
    short: &'static str,
    description: &'static str,
}

#[derive(Copy, Clone)]
enum Flag {
    Flag(FlagDescription),
    Switch(FlagDescription),
}

impl Flag {
    fn name(self) -> &'static str {
        match self {
            Flag::Flag(desc) => desc.name,
            Flag::Switch(desc) => desc.name,
        }
    }

    fn short(self) -> &'static str {
        match self {
            Flag::Flag(desc) => desc.short,
            Flag::Switch(desc) => desc.short,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Flag::Flag(desc) => desc.description,
            Flag::Switch(desc) => desc.description,
        }
    }
}

pub struct Context {
    db: rusqlite::Connection,
    client: reqwest::blocking::Client,
    config: models::Config,
    data: HashMap<&'static str, String>,
    params: Vec<String>,
}

impl Context {
    fn new() -> Context {
        Context {
            db: sqlite::new(),
            client: reqwest::blocking::Client::new(),
            config: config::load().unwrap(),
            data: HashMap::new(),
            params: Vec::new(),
        }
    }
}

struct Command {
    name: &'static str,
    aliases: Vec<&'static str>,
    description: &'static str,
    params: CommandParams,
    action: fn(ctx: &mut Context) -> utils::Result<()>,
    subcommands: Vec<Command>,
    flags: Vec<Flag>,
}

impl Command {
    fn run(&self, ctx: &mut Context, args: &[String]) {
        if args.len() == 0 {
            match (self.action)(ctx) {
                Ok(_) => {}
                Err(e) => {
                    println!("Aborting: {}", e);
                    std::process::exit(1);
                }
            };
        } else {
            let cmd = args[0].as_str();
            match cmd {
                "help" | "--help" | "-h" => self.print_help_and_exit(0),
                _ => {
                    if let Some(command) = self
                        .subcommands
                        .iter()
                        .find(|c| c.name == cmd || c.aliases.iter().find(|a| **a == cmd) != None)
                    {
                        command.run(ctx, &args[1..]);
                    } else {
                        let mut arg_iter = args.iter();

                        while let Some(a) = arg_iter.next() {
                            if a.starts_with("--") {
                                if let Some(flag) = self
                                    .flags
                                    .iter()
                                    .find(|f| f.name() == a.strip_prefix("--").unwrap())
                                {
                                    match flag {
                                        Flag::Flag(desc) => {
                                            if let Some(v) = arg_iter.next() {
                                                ctx.data.insert(desc.name, v.to_string());
                                            } else {
                                                println!(
                                                    "Aborting: flag '{}' is missing the value",
                                                    desc.name
                                                );
                                                self.print_help_and_exit(1);
                                            }
                                        }
                                        Flag::Switch(desc) => {
                                            ctx.data.insert(desc.name, String::new());
                                        }
                                    }
                                } else {
                                    println!("Aborting: unknown flag '{}'", a);
                                    self.print_help_and_exit(1);
                                }
                            } else if a.starts_with("-") {
                                if let Some(flag) = self
                                    .flags
                                    .iter()
                                    .find(|f| f.short() == a.strip_prefix("-").unwrap())
                                {
                                    match flag {
                                        Flag::Flag(desc) => {
                                            if let Some(v) = arg_iter.next() {
                                                ctx.data.insert(desc.name, v.to_string());
                                            } else {
                                                println!(
                                                    "Aborting: flag '{}' is missing the value",
                                                    desc.name
                                                );
                                                self.print_help_and_exit(1);
                                            }
                                        }
                                        Flag::Switch(desc) => {
                                            ctx.data.insert(desc.name, String::new());
                                        }
                                    }
                                } else {
                                    println!("Aborting: unknown flag '{}'", a);
                                    self.print_help_and_exit(1);
                                }
                            } else {
                                match self.params {
                                    CommandParams::None => {
                                        println!("Aborting: unexpected parameter {}", a);
                                        self.print_help_and_exit(1);
                                    }
                                    CommandParams::Single(_) => {
                                        if ctx.params.len() == 1 {
                                            println!("Aborting: too many parameters: {}", a);
                                            self.print_help_and_exit(1);
                                        }

                                        ctx.params.push(a.to_string());
                                    }
                                    CommandParams::Multi(_) => {
                                        ctx.params.push(a.to_string());
                                    }
                                }
                            }
                        }

                        match (self.action)(ctx) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Aborting: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            };
        }
    }

    fn print_help_and_exit(&self, code: i32) {
        // let mut help_str = String::with_capacity(20);
        let mut buf = Vec::new();
        buf.push(format!("\nUsage: {}", self.name));

        if self.subcommands.len() > 0 {
            buf.push(format!(" COMMAND"));
        }

        if self.flags.len() > 0 {
            buf.push(format!(" OPTIONS"));
        }

        match self.params {
            CommandParams::Single(name) => buf.push(format!(" {}", name)),
            CommandParams::Multi(name) => buf.push(format!(" {0} [{0}...]", name)),
            _ => {}
        }

        buf.push(format!("\n\n{}\n", self.description));

        if self.subcommands.len() > 0 {
            buf.push("\nCommands:".to_string());

            for c in self.subcommands.iter() {
                buf.push(format!("\n  {:<8}    {}", c.name, c.description));
            }
        }

        if self.flags.len() > 0 {
            buf.push("\nOptions:".to_string());

            for f in self.flags.iter() {
                let flag_str = format!("-{} --{}", f.short(), f.name());
                buf.push(format!("\n  {:<18}    {}", flag_str, f.description()));
            }
        }

        println!("{}", buf.join(""));
        std::process::exit(code);
    }
}

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
        ],
    };

    let mut ctx = Context::new();
    let args: Vec<String> = env::args().collect();
    app.run(&mut ctx, &args[1..]);
}

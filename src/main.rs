mod cmd;
mod config;
mod db;
mod input;
mod models;
mod output;
mod utils;

use std::collections::HashMap;
use std::env;

enum CommandParams {
    None,
    Single(&'static str),
    Multi(&'static str),
}

struct Flag {
    name: &'static str,
    // TODO: change aliases to short
    aliases: Vec<&'static str>,
    description: &'static str,
}

pub struct Context {
    db: db::sqlite::SQLiteDatabase,
    data: HashMap<&'static str, String>,
    params: Vec<String>,
}

impl Context {
    fn new() -> Context {
        Context {
            db: db::sqlite::new(),
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
    action: fn(ctx: &Context),
    subcommands: Vec<Command>,
    flags: Vec<Flag>,
}

impl Command {
    fn run(&self, ctx: &mut Context, args: &[String]) {
        if args.len() == 0 {
            (self.action)(&ctx);
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
                                    .find(|f| f.name == a.strip_prefix("--").unwrap())
                                {
                                    if let Some(v) = arg_iter.next() {
                                        ctx.data.insert(flag.name, v.to_string());
                                    } else {
                                        println!(
                                            "Aborting: flag '{}' is missing the value",
                                            flag.name
                                        );
                                        self.print_help_and_exit(1);
                                    }
                                } else {
                                    println!("Aborting: unknown flag '{}'", a);
                                    self.print_help_and_exit(1);
                                }
                            } else if a.starts_with("-") {
                                if let Some(flag) = self.flags.iter().find(|f| {
                                    f.aliases
                                        .iter()
                                        .find(|fa| **fa == a.strip_prefix("-").unwrap())
                                        != None
                                }) {
                                    if let Some(v) = arg_iter.next() {
                                        ctx.data.insert(flag.name, v.to_string());
                                    } else {
                                        println!(
                                            "Aborting: flag '{}' is missing the value",
                                            flag.name
                                        );
                                        self.print_help_and_exit(1);
                                    }
                                } else {
                                    println!("Aborting: unknown flag '{}'", a);
                                    self.print_help_and_exit(1);
                                }
                            } else {
                                ctx.params.push(a.to_string());
                            }
                        }

                        (self.action)(&ctx);
                    }
                }
            };
        }
    }

    fn print_help_and_exit(&self, code: i32) {
        let mut help_str = String::with_capacity(20);
        help_str.push_str(&format!("\nUsage: {}", self.name));

        if self.subcommands.len() > 0 {
            help_str.push_str(&format!(" COMMAND"));
        }

        if self.flags.len() > 0 {
            help_str.push_str(&format!(" OPTIONS"));
        }

        match self.params {
            CommandParams::Single(name) => help_str.push_str(&format!(" {}", name)),
            CommandParams::Multi(name) => help_str.push_str(&format!(" {0} [{0}...]", name)),
            _ => {}
        }

        help_str.push_str(&format!("\n\n{}\n", self.description));

        if self.subcommands.len() > 0 {
            help_str.push_str("\nCommands:");

            for c in self.subcommands.iter() {
                help_str.push_str(&format!("\n  {}\t{}", c.name, c.description));
            }
        }

        if self.flags.len() > 0 {
            help_str.push_str("\nOptions:");

            for f in self.flags.iter() {
                let mut alias_str = String::with_capacity(f.aliases.len() * 2);
                if f.aliases.len() > 0 {
                    for a in f.aliases.iter() {
                        alias_str.push_str(&format!("-{},", a));
                    }
                }
                help_str.push_str(&format!(
                    "\n  {} --{} string\t{}",
                    alias_str, f.name, f.description
                ));
            }
        }

        println!("{}", help_str);
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
            let s = format!("  {}  {}\n", c.name, c.description);
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
                name: "use",
                aliases: vec!["u"],
                description: "Set the default list",
                params: CommandParams::Single("LIST"),
                action: cmd::use_list,
                flags: vec![],
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
                            Flag {
                                name: "title",
                                aliases: vec!["t"],
                                description: "the list title",
                            },
                            Flag {
                                name: "desc",
                                aliases: vec!["d"],
                                description: "the list description",
                            },
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
                            Flag {
                                name: "title",
                                aliases: vec!["t"],
                                description: "the list title",
                            },
                            Flag {
                                name: "desc",
                                aliases: vec!["d"],
                                description: "the list description",
                            },
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
                params: CommandParams::None,
                action: cmd::item,
                flags: vec![Flag {
                    name: "list",
                    aliases: vec!["l"],
                    description: "the list to show",
                }],
                subcommands: vec![
                    Command {
                        name: "add",
                        aliases: vec!["a"],
                        description: "Add a new item to the list",
                        params: CommandParams::None,
                        action: cmd::item::add,
                        subcommands: vec![],
                        flags: vec![
                            Flag {
                                name: "list",
                                aliases: vec!["l"],
                                description: "the list to add the item too",
                            },
                            Flag {
                                name: "title",
                                aliases: vec!["t"],
                                description: "the item title",
                            },
                            Flag {
                                name: "desc",
                                aliases: vec!["d"],
                                description: "the item description",
                            },
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
                            Flag {
                                name: "list",
                                aliases: vec!["l"],
                                description: "the list the item is in",
                            },
                            Flag {
                                name: "title",
                                aliases: vec!["t"],
                                description: "the item title",
                            },
                            Flag {
                                name: "desc",
                                aliases: vec!["d"],
                                description: "the item description",
                            },
                        ],
                    },
                    Command {
                        name: "delete",
                        aliases: vec!["d"],
                        description: "Delete one or more items",
                        params: CommandParams::Multi("ITEM"),
                        action: cmd::item::delete,
                        subcommands: vec![],
                        flags: vec![Flag {
                            name: "list",
                            aliases: vec!["l"],
                            description: "the list the item is in",
                        }],
                    },
                ],
            },
        ],
    };

    let mut ctx = Context::new();
    let args: Vec<String> = env::args().collect();
    app.run(&mut ctx, &args[1..]);
}

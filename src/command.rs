use crate::{context::Context, utils::Result};

pub struct Command {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub description: &'static str,
    pub params: CommandParams,
    pub action: fn(ctx: &mut Context) -> Result<()>,
    pub subcommands: Vec<Command>,
    pub flags: Vec<Flag>,
}

impl Command {
    pub fn run(&self, ctx: &mut Context, args: &[String]) {
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

pub enum CommandParams {
    None,
    Single(&'static str),
    Multi(&'static str),
}

#[derive(Copy, Clone)]
pub struct FlagDescription {
    pub name: &'static str,
    pub short: &'static str,
    pub description: &'static str,
}

#[derive(Copy, Clone)]
pub enum Flag {
    Flag(FlagDescription),
    Switch(FlagDescription),
}

pub mod flags {
    pub mod flag {
        use crate::command::{Flag, FlagDescription};
        pub fn list(description: Option<&'static str>) -> Flag {
            Flag::Flag(FlagDescription {
                name: "list",
                short: "l",
                description: description.unwrap_or("The list to use"),
            })
        }

        pub fn mv(description: Option<&'static str>) -> Flag {
            Flag::Flag(FlagDescription {
                name: "move",
                short: "m",
                description: description.unwrap_or("Move exiting to list"),
            })
        }
    }

    pub mod switch {
        use crate::command::{Flag, FlagDescription};

        pub fn new(description: Option<&'static str>) -> Flag {
            Flag::Switch(FlagDescription {
                name: "new",
                short: "n",
                description: description.unwrap_or("Create new"),
            })
        }

        pub fn edit(description: Option<&'static str>) -> Flag {
            Flag::Switch(FlagDescription {
                name: "edit",
                short: "e",
                description: description.unwrap_or("Edit existing"),
            })
        }

        pub fn delete(description: Option<&'static str>) -> Flag {
            Flag::Switch(FlagDescription {
                name: "delete",
                short: "D",
                description: description.unwrap_or("Delete existing"),
            })
        }
    }
}

impl Flag {
    pub fn name(self) -> &'static str {
        match self {
            Flag::Flag(desc) => desc.name,
            Flag::Switch(desc) => desc.name,
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Flag::Flag(desc) => desc.short,
            Flag::Switch(desc) => desc.short,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Flag::Flag(desc) => desc.description,
            Flag::Switch(desc) => desc.description,
        }
    }
}

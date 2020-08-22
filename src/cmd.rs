pub mod list {
    use crate::db;

    pub fn description() -> &'static str {
        "Manage lists"
    }

    pub fn process(args: &[String]) {
        if args.len() == 0 {
            let lists = db::get_lists();

            println!("NAME\tDESCRIPTION");
            for l in lists.iter() {
                println!("{}\t{}", l.title, l.description);
            }
            return;
        }

        match args[0].as_str() {
            "create" => {}
            "edit" => {}
            "delete" => {}
            "show" => {}
            "help" => print_help_and_exit(0),
            _ => print_help_and_exit(1),
        }
    }

    fn print_help_and_exit(code: i32) {
        println!("procrast list COMMAND");
        std::process::exit(code);
    }
}

pub mod item {
    pub fn description() -> &'static str {
        "Manage items"
    }

    pub fn process(args: &[String]) {
        if args.len() == 0 {
            print_help_and_exit(1);
        }

        match args[0].as_str() {
            "add" => {}
            "edit" => {}
            "delete" => {}
            "help" => print_help_and_exit(0),
            _ => print_help_and_exit(1),
        }
    }

    fn print_help_and_exit(code: i32) {
        println!("procrast item COMMAND");
        std::process::exit(code);
    }
}

fn print_help_and_exit(code: i32) {
    println!(
        "

Usage: procrast COMMAND

A cli for managing your procrastination

Commands:
  list  {}
  item  {}
",
        list::description(),
        item::description()
    );

    std::process::exit(code);
}

pub fn process(args: &[String]) {
    if args.len() == 0 {
        print_help_and_exit(1);
    }

    match args[0].as_str() {
        "list" => list::process(&args[1..]),
        "item" => item::process(&args[1..]),
        "help" | "--help" | "-h" => print_help_and_exit(0),
        _ => print_help_and_exit(1),
    };
}

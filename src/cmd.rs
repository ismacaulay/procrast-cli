fn print_help_and_exit(code: i32) {
    // TODO: implement printing help
    println!("procrast <command>");

    std::process::exit(code);
}

mod cmd {
    pub mod item {
        fn description() -> &'static str {
            return "Manage items";
        }

        fn print_item_help_and_exit(code: i32) {
            // TODO: implement printing help
            println!("procrast item <command>");
            std::process::exit(code);
        }

        pub fn process(args: &[String]) -> bool {
            println!("item cmd: {:?}", args);
            if args.len() == 0 {
                print_item_help_and_exit(0)
            }

            match args[0].as_str() {
                "add" => {
                    println!("ADD!");
                }
                _ => {
                    print_item_help_and_exit(1);
                    return false;
                }
            }

            return true;
        }
    }
}

fn unknown(cmd: &String) -> bool {
    println!("Unknown command: {}", cmd);
    false
}

pub fn process(args: &[String]) {
    if args.len() == 0 {
        print_help_and_exit(1);
    }

    match args[0].as_str() {
        "item" => cmd::item::process(&args[1..]),
        _ => unknown(&args[0]),
    };
}

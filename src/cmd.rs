pub mod list {
    use crate::{db, input, models};

    pub fn description() -> &'static str {
        "Manage lists"
    }

    fn split_text_into_title_desc(text: &String) -> Option<(Option<String>, Option<String>)> {
        let trimmed = text.trim();
        if trimmed.len() > 0 {
            // TODO: Handle \r\n
            let mut iter = trimmed.splitn(2, '\n');
            let title = iter.next().map(|s| String::from(s.trim()));
            let description = iter.next().map(|s| String::from(s.trim()));

            return Some((title, description));
        }

        return None;
    }

    pub fn process(args: &[String]) {
        if args.len() == 0 {
            let lists = db::get_lists();

            println!("ID\tNAME\tDESCRIPTION");
            for l in lists.iter() {
                println!("{}\t{}\t{}", l.id, l.title, l.description);
            }
            return;
        }

        match args[0].as_str() {
            "create" => {
                let mut title: Option<String> = None;
                let mut description: Option<String> = None;

                if args.len() == 1 {
                    let text = input::get_file_input(None);
                    if let Some(result) = split_text_into_title_desc(&text) {
                        let (t, d) = result;
                        title = t;
                        description = d;
                    }
                } else {
                    let (t, d) = parse_name_and_description_options(&args[1..]);
                    title = t;
                    description = d;
                }

                if title == None {
                    println!("Aborting: no list name");
                } else {
                    if description == None {
                        db::create_list(&title.unwrap(), &String::from(""));
                    } else {
                        db::create_list(&title.unwrap(), &description.unwrap());
                    }
                }
            }
            "edit" => {
                if args.len() == 1 {
                    // TODO: use current list and show editor
                } else if args.len() == 2 {
                    let list_id = &args[1];

                    if let Some(list) = db::find_list_by_title(list_id).as_mut() {
                        edit_list(list);
                    } else if let Some(list) = db::find_list_by_id(list_id).as_mut() {
                        edit_list(list);
                    } else {
                        println!("Not Found: {:?}", list_id);
                    }
                } else {
                    let list_id = &args[1];

                    if let Some(list) = db::find_list_by_title(list_id).as_mut() {
                        let (title, description) = parse_name_and_description_options(&args[2..]);
                        update_list_with_title_and_description(list, title, description);
                    } else if let Some(list) = db::find_list_by_id(list_id).as_mut() {
                        let (title, description) = parse_name_and_description_options(&args[2..]);
                        update_list_with_title_and_description(list, title, description);
                    } else {
                        println!("Not Found: {:?}", list_id);
                    }
                }
            }
            "delete" => {
                if args.len() == 1 {
                    // TODO: use current list
                } else if args.len() == 2 {
                    let list_id = &args[1];

                    if let Some(list) = db::find_list_by_title(list_id).as_mut() {
                        delete_list(list);
                    } else if let Some(list) = db::find_list_by_id(list_id).as_mut() {
                        delete_list(list);
                    } else {
                        println!("Not Found: {:?}", list_id);
                    }
                }
            }
            "show" => {}
            "help" | "--help" | "-h" => print_help_and_exit(0),
            _ => print_help_and_exit(1),
        }
    }

    fn print_help_and_exit(code: i32) {
        println!("procrast list COMMAND");
        std::process::exit(code);
    }

    fn edit_list(list: &mut models::List) {
        let current = vec![list.title.clone(), list.description.clone()].join("\n");
        let update = input::get_file_input(Some(&current));
        if let Some(result) = split_text_into_title_desc(&update) {
            let (title, mut description) = result;

            if description == None {
                description = Some(String::from(""));
            }

            update_list_with_title_and_description(list, title, description);
        }
    }

    fn delete_list(list: &models::List) {
        println!("Are you sure you want to delete list '{}'?", list.title);
        println!("This cannot be undone!");
        print!("Enter ther name of the list to confirm:");
        let result = input::get_stdin_input();
        if result == list.title {
            db::delete_list(list);
        } else {
            println!("Aborting: Entered name does not match {}", list.title);
        }
    }

    fn update_list_with_title_and_description(
        list: &mut models::List,
        title: Option<String>,
        description: Option<String>,
    ) {
        if let Some(t) = title {
            list.title = t;
        }

        if let Some(d) = description {
            list.description = d;
        }

        db::update_list(list)
    }

    fn parse_name_and_description_options(args: &[String]) -> (Option<String>, Option<String>) {
        let mut idx = 0;
        let mut title: Option<String> = None;
        let mut description: Option<String> = None;

        while idx < args.len() {
            match args[idx].as_str() {
                "-n" => {
                    idx += 1;

                    if idx < args.len() {
                        title = Some(args[idx].clone());
                        idx += 1;
                    } else {
                        print_help_and_exit(1);
                    }
                }
                "-d" => {
                    idx += 1;

                    if idx < args.len() {
                        description = Some(args[idx].clone());
                        idx += 1;
                    } else {
                        print_help_and_exit(1);
                    }
                }
                _ => {
                    println!("Unknown option '{}'", args[idx]);
                    print_help_and_exit(1);
                }
            }
        }

        return (title, description);
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

use crate::{db::Database, output::TablePrinter, Context};

pub fn use_list(ctx: &Context) {
    if ctx.params.len() != 0 {
        let list_id = &ctx.params[0];
        if let Some(list) = ctx.db.find_list(list_id) {
            ctx.db.set_current_list(&list.id.to_string());
        } else {
            println!("Aborting: Could not find list '{}'", list_id)
        }
    }
}

pub fn list(ctx: &Context) {
    let lists = ctx.db.get_lists();

    // TODO: eventually current will be a uuid instead of i32
    let current: i32 = match ctx.db.get_current_list() {
        Some(c) => c.parse::<i32>().unwrap(),
        None => -1,
    };

    let mut printer = TablePrinter::new(vec!["ID".to_string(), "TITLE".to_string()]);
    for l in lists.iter() {
        printer
            .add_row(vec![
                format!(
                    "{}{}",
                    l.id.to_string(),
                    if l.id == current { "*" } else { "" }
                ),
                l.title.clone(),
            ])
            .expect("Failed to add row to printer");
    }
    printer.print();
}

pub fn item(ctx: &Context) {
    let list_id: Option<String>;

    if let Some(list) = ctx.data.get("list") {
        if let Some(l) = ctx.db.find_list(list) {
            list_id = Some(l.id.to_string());
        } else {
            println!("Aborting. Unknown list {}", list);
            std::process::exit(1);
        }
    } else {
        list_id = ctx.db.get_current_list();
        if list_id.is_none() {
            println!("Aborting. No list specified");
            std::process::exit(1);
        }
    }

    // TODO: check for -l argument to specify the list
    if let Some(id) = list_id {
        let items = ctx.db.get_items(&id);

        let mut printer = TablePrinter::new(vec!["ID".to_string(), "TITLE".to_string()]);
        for i in items.iter() {
            printer
                .add_row(vec![i.id.to_string(), i.title.clone()])
                .expect("Failed to add row to printer");
        }
        printer.print();
    } else {
        println!("No list selected");
    }
}

pub mod list {
    use crate::{db::Database, input, utils, Context};

    pub fn create(ctx: &Context) {
        let mut title: Option<String> = None;
        let mut description: Option<String> = None;

        if let Some(value) = ctx.data.get("title") {
            title = Some(String::from(value));
        }

        if let Some(value) = ctx.data.get("desc") {
            description = Some(String::from(value));
        }

        if title == None && description == None {
            // get input from file
            let text = input::get_file_input(None);
            if let Some(result) = utils::split_text_into_title_desc(&text) {
                let (t, d) = result;
                title = t;
                description = d;
            }
        } else if title == None && description != None {
            // get input for title from stdin
            print!("Please enter the list title: ");
            let text = input::get_stdin_input();
            if text.len() > 0 {
                title = Some(text);
            }
        } else if title != None && description == None {
            // save title with empty description
            description = Some(String::from(""));
        }

        if title == None {
            println!("Aborting: no list title");
            std::process::exit(1);
        }

        ctx.db.create_list(&title.unwrap(), &description.unwrap());
    }

    pub fn show(ctx: &Context) {
        let list_id: String;
        if ctx.params.len() == 0 {
            if let Some(id) = ctx.db.get_current_list() {
                list_id = id;
            } else {
                println!("Aborting. No list specified");
                std::process::exit(1);
            }
        } else {
            list_id = ctx.params[0].clone();
        }

        if let Some(list) = ctx.db.find_list(&list_id) {
            println!("{}: {}", list.id, list.title);

            if list.description.len() > 0 {
                println!("\n{}\n", list.description);
            }
        }
    }

    pub fn edit(ctx: &Context) {
        if ctx.params.len() == 0 {
            // TODO: use current list and show editor
        } else if ctx.params.len() == 1 {
            let list_id = &ctx.params[0];
            if let Some(list) = ctx.db.find_list(list_id).as_mut() {
                let mut title: Option<String> = None;
                let mut description: Option<String> = None;

                if let Some(value) = ctx.data.get("title") {
                    title = Some(String::from(value));
                }

                if let Some(value) = ctx.data.get("desc") {
                    description = Some(String::from(value));
                }

                if title == None && description == None {
                    // get input from file
                    let current = vec![list.title.clone(), list.description.clone()].join("\n");
                    let text = input::get_file_input(Some(&current));
                    if let Some(result) = utils::split_text_into_title_desc(&text) {
                        let (t, d) = result;
                        title = t;
                        description = d;
                    }
                }

                if title.is_some() || description.is_some() {
                    if let Some(t) = title {
                        list.title = t;
                    }

                    if let Some(d) = description {
                        list.description = d;
                    }

                    ctx.db.update_list(list);
                }
            } else {
                println!("Aborting: Could not find list '{}'", list_id)
            }
        }
    }

    pub fn delete(ctx: &Context) {
        if ctx.params.len() == 0 {
            // TODO: use current list
        } else {
            for p in ctx.params.iter() {
                if let Some(list) = ctx.db.find_list(p) {
                    println!("Are you sure you want to delete list '{}'?", list.title);
                    println!("This cannot be undone!");
                    print!("Enter ther name of the list to confirm: ");
                    let result = input::get_stdin_input();
                    if result == list.title {
                        let list_id = list.id.to_string();
                        for item in ctx.db.get_items(&list_id) {
                            ctx.db.delete_item(&list_id, &item);
                        }

                        if let Some(current) = ctx.db.get_current_list() {
                            if list_id == current {
                                ctx.db.set_current_list(&String::new());
                            }
                        }

                        ctx.db.delete_list(&list);
                    } else {
                        println!(
                            "Skipping '{}'. Entered title does not match {}",
                            p, list.title
                        );
                    }
                } else {
                    println!("Skipping '{}'. Not found!", p);
                }
            }
        }
    }
}

pub mod item {
    use crate::{db::Database, input, utils, Context};

    pub fn add(ctx: &Context) {
        let list_id: Option<String>;
        let mut title: Option<String> = None;
        let mut description: Option<String> = None;

        if let Some(list) = ctx.data.get("list") {
            if let Some(l) = ctx.db.find_list(list) {
                list_id = Some(l.id.to_string());
            } else {
                println!("Aborting. Unknown list {}", list);
                std::process::exit(1);
            }
        } else {
            list_id = ctx.db.get_current_list();
            if list_id.is_none() {
                println!("Aborting. No list specified");
                std::process::exit(1);
            }
        }

        if let Some(value) = ctx.data.get("title") {
            title = Some(String::from(value));
        }

        if let Some(value) = ctx.data.get("desc") {
            description = Some(String::from(value));
        }

        if title == None && description == None {
            // get input from file
            let text = input::get_file_input(None);
            if let Some(result) = utils::split_text_into_title_desc(&text) {
                let (t, d) = result;
                title = t;
                description = d;
            }
        } else if title == None && description != None {
            // get input for title from stdin
            print!("Please enter the item title: ");
            let text = input::get_stdin_input();
            if text.len() > 0 {
                title = Some(text);
            }
        } else if title != None && description == None {
            // save title with empty description
            description = Some(String::from(""));
        }

        if title == None {
            println!("Aborting: no item title");
            std::process::exit(1);
        }

        ctx.db
            .create_item(&list_id.unwrap(), &title.unwrap(), &description.unwrap());
    }

    pub fn show(ctx: &Context) {
        if ctx.params.len() == 0 {
            println!("Aborting: no item specified");
            std::process::exit(1);
        } else if ctx.params.len() == 1 {
            let list_id: String;

            if let Some(list) = ctx.data.get("list") {
                if let Some(l) = ctx.db.find_list(list) {
                    list_id = l.id.to_string();
                } else {
                    println!("Aborting. Unknown list {}", list);
                    std::process::exit(1);
                }
            } else {
                if let Some(l) = ctx.db.get_current_list() {
                    list_id = l;
                } else {
                    println!("Aborting. No list specified");
                    std::process::exit(1);
                }
            }

            let item_id = &ctx.params[0];
            if let Some(item) = ctx.db.get_item(&list_id, item_id).as_mut() {
                println!("{}: {}", item.id, item.title);

                if item.description.len() > 0 {
                    println!("\n{}\n", item.description);
                }
            }
        }
    }

    pub fn edit(ctx: &Context) {
        if ctx.params.len() == 0 {
            println!("Aborting: no item specified");
            std::process::exit(1);
        } else if ctx.params.len() == 1 {
            let list_id: String;

            if let Some(list) = ctx.data.get("list") {
                if let Some(l) = ctx.db.find_list(list) {
                    list_id = l.id.to_string();
                } else {
                    println!("Aborting. Unknown list {}", list);
                    std::process::exit(1);
                }
            } else {
                if let Some(l) = ctx.db.get_current_list() {
                    list_id = l;
                } else {
                    println!("Aborting. No list specified");
                    std::process::exit(1);
                }
            }

            let item_id = &ctx.params[0];
            if let Some(item) = ctx.db.get_item(&list_id, item_id).as_mut() {
                let mut title: Option<String> = None;
                let mut description: Option<String> = None;

                if let Some(value) = ctx.data.get("title") {
                    title = Some(String::from(value));
                }

                if let Some(value) = ctx.data.get("desc") {
                    description = Some(String::from(value));
                }

                if title == None && description == None {
                    // get input from file
                    let current = vec![item.title.clone(), item.description.clone()].join("\n");
                    let text = input::get_file_input(Some(&current));
                    if let Some(result) = utils::split_text_into_title_desc(&text) {
                        let (t, d) = result;
                        title = t;
                        description = d;
                    }
                }

                if title.is_some() || description.is_some() {
                    if let Some(t) = title {
                        item.title = t;
                    }

                    if let Some(d) = description {
                        item.description = d;
                    }

                    ctx.db.update_item(&list_id, item);
                }
            } else {
                println!("Aborting: Could not find item '{}'", list_id)
            }
        }
    }

    pub fn delete(ctx: &Context) {
        let list_id: String;

        if let Some(list) = ctx.data.get("list") {
            if let Some(l) = ctx.db.find_list(list) {
                list_id = l.id.to_string();
            } else {
                println!("Aborting. Unknown list {}", list);
                std::process::exit(1);
            }
        } else {
            if let Some(l) = ctx.db.get_current_list() {
                list_id = l;
            } else {
                println!("Aborting. No list specified");
                std::process::exit(1);
            }
        }

        for p in ctx.params.iter() {
            if let Some(item) = ctx.db.get_item(&list_id, p) {
                ctx.db.delete_item(&list_id, &item);
            } else {
                println!("Skipping '{}'. Not found in list '{}'!", p, list_id);
            }
        }
    }
}

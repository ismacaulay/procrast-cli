use crate::{config, db, CommandCtx};

pub fn use_list(ctx: &CommandCtx) {
    if ctx.params.len() != 0 {
        let list_id = &ctx.params[0];
        if let Some(list) = db::find_list(list_id) {
            config::set_current_list(&list.id.to_string());
        } else {
            println!("Aborting: Could not find list '{}'", list_id)
        }
    }
}

pub fn list(_: &CommandCtx) {
    let lists = db::get_lists();

    println!("ID\tNAME\tDESCRIPTION");
    for l in lists.iter() {
        println!("{}\t{}\t{}", l.id, l.title, l.description);
    }
}

pub fn item(_: &CommandCtx) {
    // TODO: show list item for selected list?
}

pub mod list {
    use crate::{db, input, CommandCtx};

    pub fn create(ctx: &CommandCtx) {
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
            if let Some(result) = split_text_into_title_desc(&text) {
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

        db::create_list(&title.unwrap(), &description.unwrap());
    }

    pub fn edit(ctx: &CommandCtx) {
        if ctx.params.len() == 0 {
            // TODO: use current list and show editor
        } else if ctx.params.len() == 1 {
            let list_id = &ctx.params[0];
            if let Some(list) = db::find_list(list_id).as_mut() {
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
                    if let Some(result) = split_text_into_title_desc(&text) {
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

                    db::update_list(list);
                }
            } else {
                println!("Aborting: Could not find list '{}'", list_id)
            }
        }
    }

    pub fn delete(ctx: &CommandCtx) {
        if ctx.params.len() == 0 {
            // TODO: use current list
        } else {
            for p in ctx.params.iter() {
                if let Some(list) = db::find_list(p) {
                    println!("Are you sure you want to delete list '{}'?", list.title);
                    println!("This cannot be undone!");
                    print!("Enter ther name of the list to confirm: ");
                    let result = input::get_stdin_input();
                    if result == list.title {
                        db::delete_list(&list);
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
}

pub mod item {}

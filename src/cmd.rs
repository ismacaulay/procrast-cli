use crate::{log, models, output::TablePrinter, sqlite, Context};
use std::result;

pub type Result<T, E = String> = result::Result<T, E>;

fn get_current_list(ctx: &Context) -> Result<uuid::Uuid> {
    match sqlite::get_current_list(&ctx.db) {
        Ok(current) => Ok(current),
        Err(e) => {
            log::println(format!("sqlite: {}", e));
            return Err("No list in use. See the use command for help".to_string());
        }
    }
}

fn update_item(ctx: &Context, item: &models::Item) -> Result<()> {
    match sqlite::update_item(&ctx.db, item) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Failed to update item {}", item.id)),
    }
}

fn find_list_by_id(ctx: &Context, id: &String) -> Result<models::List> {
    match sqlite::find_list_by_id(&ctx.db, id) {
        Ok(list) => Ok(list),
        Err(e) => {
            log::println(format!("sqlite: {}", e.to_string()));
            Err(format!("Failed to find list with id: {}", id))
        }
    }
}

fn find_list_by_uuid(ctx: &Context, uuid: &uuid::Uuid) -> Result<models::List> {
    match sqlite::find_list_by_uuid(&ctx.db, uuid) {
        Ok(list) => Ok(list),
        Err(_) => Err(format!(
            "Failed to find list with uuid: {}",
            uuid.to_hyphenated().to_string()
        )),
    }
}

fn find_item_by_id(ctx: &Context, list_uuid: &uuid::Uuid, id: &String) -> Result<models::Item> {
    match sqlite::get_item(&ctx.db, list_uuid, id) {
        Ok(item) => Ok(item),
        Err(_) => Err(format!("Failed to find item with id: {}", id)),
    }
}

fn find_list_or_current(ctx: &Context) -> Result<models::List> {
    if let Some(id) = ctx.data.get("list") {
        return find_list_by_id(ctx, id);
    }
    return find_list_by_uuid(ctx, &get_current_list(ctx)?);
}

pub fn use_list(ctx: &mut Context) -> Result<()> {
    if ctx.params.len() != 0 {
        let list_id = &ctx.params[0];
        let list = find_list_by_id(ctx, list_id)?;
        return match sqlite::set_current_list(&ctx.db, Some(&list.uuid)) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to set current list".to_string()),
        };
    }

    Err("No list id provided".to_string())
}

pub fn list(ctx: &mut Context) -> Result<()> {
    let lists = match sqlite::get_lists(&ctx.db) {
        Ok(lists) => lists,
        Err(_) => return Err("Failed to retrieve lists".to_string()),
    };
    let current = match get_current_list(ctx) {
        Ok(uuid) => Some(uuid),
        Err(_) => None,
    };

    let mut printer = TablePrinter::new(vec!["ID".to_string(), "TITLE".to_string()]);
    for l in lists.iter() {
        printer
            .add_row(vec![
                format!(
                    "{}{}",
                    l.id.to_string(),
                    if current.is_some() && l.uuid == current.unwrap() {
                        "*"
                    } else {
                        ""
                    }
                ),
                l.title.clone(),
            ])
            .expect("Failed to add row to printer");
    }
    printer.print();

    Ok(())
}

pub fn item(ctx: &mut Context) -> Result<()> {
    let list_id: uuid::Uuid;

    if let Some(list) = ctx.data.get("list") {
        list_id = find_list_by_id(ctx, list)?.uuid;
    } else {
        list_id = get_current_list(&ctx)?;
    }

    if ctx.params.len() == 0 {
        let items = match sqlite::get_items(&ctx.db, &list_id) {
            Ok(items) => items,
            Err(_) => return Err(format!("Failed to get items for list {}", list_id)),
        };

        let mut printer = TablePrinter::new(vec![
            "ID".to_string(),
            "STATE".to_string(),
            "TITLE".to_string(),
        ]);
        for i in items.iter() {
            printer
                .add_row(vec![
                    i.id.to_string(),
                    if i.state == 0 {
                        "-".to_string()
                    } else {
                        "x".to_string()
                    },
                    i.title.clone(),
                ])
                .expect("Failed to add row to printer");
        }
        printer.print();
    } else {
        let complete_flag = ctx.data.get("complete");
        let incomplete_flag = ctx.data.get("incomplete");

        if complete_flag.is_some() && incomplete_flag.is_some() {
            println!("Aborting: an item can not be complete and incomplete, this is ambiguous!");
            std::process::exit(1);
        }

        let item_id = &ctx.params[0];
        match sqlite::get_item(&ctx.db, &list_id, item_id).as_mut() {
            Ok(item) => {
                if let Some(_) = complete_flag {
                    item.state = 1;
                    update_item(ctx, item)?;
                } else if let Some(_) = incomplete_flag {
                    item.state = 0;
                    update_item(ctx, item)?;
                } else {
                    println!("{}: {}", item.id, item.title);

                    println!(
                        "\nStatus: {}",
                        if item.state == 0 {
                            "INCOMPLETE"
                        } else {
                            "COMPLETE"
                        }
                    );

                    if item.description.len() > 0 {
                        println!("\n{}\n", item.description);
                    }
                }
            }
            Err(_) => return Err(format!("Failed to get item {}", item_id)),
        }
    }

    Ok(())
}

pub mod list {
    use crate::{
        cmd::{self, Result},
        input, log, models, sqlite, utils, Context,
    };

    pub fn create(ctx: &mut Context) -> Result<()> {
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

        let now = utils::now();

        match sqlite::transaction(&mut ctx.db, |tx| {
            let list_id = sqlite::get_next_list_id(&tx)?;
            let list = models::List {
                uuid: uuid::Uuid::new_v4(),
                id: list_id,
                title: title.as_ref().unwrap().clone(),
                description: description.as_ref().unwrap().clone(),
                created: now,
                modified: now,
                next_item_id: 1,
            };
            sqlite::create_list(&tx, &list)?;
            sqlite::set_next_list_id(&tx, list_id + 1)?;
            Ok(())
        }) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to create list".to_string()),
        }
    }

    pub fn show(ctx: &mut Context) -> Result<()> {
        let list: models::List;
        if ctx.params.len() == 0 {
            let list_uuid = cmd::get_current_list(ctx)?;
            list = match sqlite::find_list_by_uuid(&ctx.db, &list_uuid) {
                Ok(list) => list,
                Err(_) => return Err(format!("Failed to find list with uuid: {}", list_uuid)),
            };
        } else {
            list = cmd::find_list_by_id(ctx, &ctx.params[0])?;
        }

        println!("{}: {}", list.id, list.title);
        if list.description.len() > 0 {
            println!("\n{}\n", list.description);
        }

        Ok(())
    }

    pub fn edit(ctx: &mut Context) -> Result<()> {
        if ctx.params.len() == 0 {
            // TODO: use current list and show editor
            return Err("TODO: allow edit to use current list".to_string());
        } else if ctx.params.len() == 1 {
            let mut list = cmd::find_list_by_id(ctx, &ctx.params[0])?;

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
                let current = vec![
                    list.title.clone(),
                    String::from(""),
                    list.description.clone(),
                ]
                .join("\n");
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

                list.modified = utils::now();
                match sqlite::update_list(&ctx.db, &list) {
                    Ok(_) => {}
                    Err(e) => {
                        log::println(format!("sqlite: {}", e));
                        return Err(format!("Failed to update list: {}", list.id));
                    }
                };
            }
        }
        Ok(())
    }

    pub fn delete(ctx: &mut Context) -> Result<()> {
        if ctx.params.len() == 0 {
            return Err("TODO: allow edit to use current list".to_string());
        } else {
            for p in ctx.params.iter() {
                let list = match cmd::find_list_by_id(ctx, p) {
                    Ok(list) => list,
                    Err(_) => {
                        println!("Skipping '{}'. Not found!", p);
                        continue;
                    }
                };

                println!("Are you sure you want to delete list '{}'?", list.title);
                println!("This cannot be undone!");
                print!("Enter ther name of the list to confirm: ");
                let result = input::get_stdin_input();
                if result == list.title {
                    match sqlite::transaction(&mut ctx.db, |tx| {
                        for item in sqlite::get_items(tx, &list.uuid)?.iter() {
                            sqlite::delete_item(tx, item)?;
                        }

                        match sqlite::get_current_list(tx) {
                            Ok(uuid) => {
                                if uuid == list.uuid {
                                    sqlite::set_current_list(tx, None)?;
                                }
                            }
                            Err(_) => {}
                        }

                        sqlite::delete_list(tx, &list)?;
                        Ok(())
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            log::println(format!("sqlite: {}", e));
                            println!("Failed to delete list: {}", list.id);
                        }
                    };
                } else {
                    println!(
                        "Skipping '{}'. Entered title does not match {}",
                        p, list.title
                    );
                }
            }
        }
        Ok(())
    }
}

pub mod item {
    use crate::{
        cmd::{self, Result},
        input, models, sqlite, utils, Context,
    };

    pub fn add(ctx: &mut Context) -> Result<()> {
        let mut list: models::List;
        let mut title: Option<String> = None;
        let mut description: Option<String> = None;

        if let Some(id) = ctx.data.get("list") {
            list = cmd::find_list_by_id(ctx, id)?;
        } else {
            list = cmd::find_list_by_uuid(ctx, &cmd::get_current_list(ctx)?)?;
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
            return Err("No item title".to_string());
        }

        match sqlite::transaction(&mut ctx.db, |tx| {
            let now = utils::now();
            let item = models::Item {
                uuid: uuid::Uuid::new_v4(),
                id: list.next_item_id,
                title: title.as_ref().unwrap().clone(),
                description: description.as_ref().unwrap().clone(),
                state: 0,
                created: now,
                modified: now,
                list_uuid: list.uuid,
            };
            list.next_item_id += 1;

            sqlite::update_list(tx, &list)?;
            sqlite::create_item(tx, &item)?;
            Ok(())
        }) {
            Ok(_) => {}
            Err(_) => return Err("Failed to create list".to_string()),
        };

        Ok(())
    }

    pub fn show(ctx: &mut Context) -> Result<()> {
        if ctx.params.len() == 0 {
            return Err("No item specified".to_string());
        } else if ctx.params.len() == 1 {
            let list = cmd::find_list_or_current(ctx)?;
            let item = cmd::find_item_by_id(ctx, &list.uuid, &ctx.params[0])?;

            println!("{}: {}", item.id, item.title);

            if item.description.len() > 0 {
                println!("\n{}\n", item.description);
            }
        }
        Ok(())
    }

    pub fn edit(ctx: &mut Context) -> Result<()> {
        if ctx.params.len() == 0 {
            return Err("No item specified".to_string());
        } else if ctx.params.len() == 1 {
            let list = cmd::find_list_or_current(ctx)?;
            let mut item = cmd::find_item_by_id(ctx, &list.uuid, &ctx.params[0])?;

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
                let current = vec![
                    item.title.clone(),
                    String::from(""),
                    item.description.clone(),
                ]
                .join("\n");
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

                match sqlite::update_item(&ctx.db, &item) {
                    Ok(_) => {}
                    Err(_) => return Err(format!("Failed to update item: {}", item.id)),
                };
            }
        }
        Ok(())
    }

    pub fn delete(ctx: &mut Context) -> Result<()> {
        let list = cmd::find_list_or_current(ctx)?;

        for id in ctx.params.iter() {
            match sqlite::get_item(&ctx.db, &list.uuid, id) {
                Ok(item) => match sqlite::delete_item(&ctx.db, &item) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to delete item: {}", item.id);
                    }
                },
                Err(_) => {
                    println!("Skipping '{}'. Not found in list '{}'!", id, list.id);
                }
            }
        }

        Ok(())
    }
}

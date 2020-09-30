use crate::{
    cmd::{self, Result},
    input, log,
    models::{self, CMD_LIST_CREATE, CMD_LIST_DELETE, CMD_LIST_UPDATE},
    sqlite, utils, Context,
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
    }

    if title == None {
        println!("Aborting: no list title");
        std::process::exit(1);
    }

    if description == None {
        description = Some(String::from(""));
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
        sqlite::create_history(
            tx,
            &models::History {
                uuid: uuid::Uuid::new_v4(),
                command: CMD_LIST_CREATE.to_string(),
                state: utils::encode_history_state(&models::CmdListState {
                    uuid: list.uuid,
                    title: list.title.clone(),
                    description: list.description.clone(),
                    created: list.created,
                    modified: list.modified,
                })?,
                created: now,
                synced: false,
            },
        )?;
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
            let now = utils::now();

            if let Some(t) = title {
                list.title = t;
            }

            if let Some(d) = description {
                list.description = d;
            }

            list.modified = now;

            sqlite::transaction(&mut ctx.db, |tx| {
                sqlite::update_list(tx, &list)?;

                sqlite::create_history(
                    tx,
                    &models::History {
                        uuid: uuid::Uuid::new_v4(),
                        command: CMD_LIST_UPDATE.to_string(),
                        state: utils::encode_history_state(&models::CmdListState {
                            uuid: list.uuid,
                            title: list.title.clone(),
                            description: list.description.clone(),
                            created: list.created,
                            modified: list.modified,
                        })?,
                        created: now,
                        synced: false,
                    },
                )?;
                Ok(())
            })?;
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
                    sqlite::create_history(
                        tx,
                        &models::History {
                            uuid: uuid::Uuid::new_v4(),
                            command: CMD_LIST_DELETE.to_string(),
                            state: utils::encode_history_state(&models::CmdDeleteState {
                                uuid: list.uuid,
                            })?,
                            created: utils::now(),
                            synced: false,
                        },
                    )?;
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

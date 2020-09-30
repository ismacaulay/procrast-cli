use crate::{
    cmd::{self, Result},
    input,
    models::{self, CMD_ITEM_CREATE, CMD_ITEM_DELETE, CMD_ITEM_UPDATE},
    sqlite, utils, Context,
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
    }

    if title == None {
        return Err("No item title".to_string());
    }

    if description == None {
        description = Some(String::from(""));
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
        sqlite::create_history(
            tx,
            &models::History {
                uuid: uuid::Uuid::new_v4(),
                command: CMD_ITEM_CREATE.to_string(),
                state: utils::encode_history_state(&models::CmdItemState {
                    uuid: item.uuid,
                    title: item.title.clone(),
                    description: item.description.clone(),
                    state: item.state,
                    created: item.created,
                    modified: item.modified,
                    list_uuid: item.list_uuid,
                })?,
                timestamp: now,
                synced: false,
            },
        )?;
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
            let now = utils::now();

            if let Some(t) = title {
                item.title = t;
            }

            if let Some(d) = description {
                item.description = d;
            }

            item.modified = now;

            sqlite::transaction(&mut ctx.db, |tx| {
                sqlite::update_item(tx, &item)?;

                sqlite::create_history(
                    tx,
                    &models::History {
                        uuid: uuid::Uuid::new_v4(),
                        command: CMD_ITEM_UPDATE.to_string(),
                        state: utils::encode_history_state(&models::CmdItemState {
                            uuid: item.uuid,
                            title: item.title.clone(),
                            description: item.description.clone(),
                            state: item.state,
                            created: item.created,
                            modified: item.modified,
                            list_uuid: item.list_uuid,
                        })?,
                        timestamp: now,
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
    let list = cmd::find_list_or_current(ctx)?;
    for id in ctx.params.iter() {
        if let Err(e) = sqlite::transaction(&mut ctx.db, |tx| {
            let item = sqlite::get_item(tx, &list.uuid, id)?;
            sqlite::delete_item(tx, &item)?;

            sqlite::create_history(
                tx,
                &models::History {
                    uuid: uuid::Uuid::new_v4(),
                    command: CMD_ITEM_DELETE.to_string(),
                    state: utils::encode_history_state(&models::CmdDeleteState {
                        uuid: item.uuid,
                    })?,
                    timestamp: utils::now(),
                    synced: false,
                },
            )?;

            Ok(())
        }) {
            println!("Skipping {}: Failed to delete: {}", id, e);
        }
    }

    Ok(())
}

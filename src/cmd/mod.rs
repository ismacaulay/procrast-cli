pub mod item;
pub mod list;

use crate::{log, models, output::TablePrinter, sqlite, utils::Result, Context};

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

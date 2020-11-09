use crate::{
    cmd,
    command::{flags, Command, CommandParams},
    context::Context,
    input,
    models::{self, CMD_NOTE_CREATE, CMD_NOTE_DELETE, CMD_NOTE_MOVE, CMD_NOTE_UPDATE},
    output, sqlite,
    utils::{self, Result},
};

pub fn command() -> Command {
    Command {
        name: "notes",
        aliases: vec![],
        description: "Manage notes",
        params: CommandParams::Multi("NOTES"),
        action: handle_notes_command,
        flags: vec![
            flags::flag::list(None),
            flags::switch::new(Some("Create a new note")),
            flags::switch::edit(Some("Edit an existing note")),
            flags::switch::delete(Some("Delete existing notes")),
            flags::flag::mv(Some("Move notes to list")),
        ],
        subcommands: vec![],
    }
}

fn handle_notes_command(ctx: &mut Context) -> Result<()> {
    let new_flag = ctx.data.get("new");
    let edit_flag = ctx.data.get("edit");
    let delete_flag = ctx.data.get("delete");
    let mv_flag = ctx.data.get("move");

    let num_flags = vec![new_flag, edit_flag, delete_flag, mv_flag]
        .iter()
        .filter(|s| s.is_some())
        .count();

    match num_flags {
        0 => match ctx.params.len() {
            0 => list(ctx),
            1 => show(ctx),
            _ => Err("Too many id parameters specified for command".to_string()),
        },
        1 => {
            if new_flag.is_some() {
                return create(ctx);
            }

            if edit_flag.is_some() {
                return match ctx.params.len() {
                    0 => Err("No note parameter specified".to_string()),
                    1 => edit(ctx),
                    _ => Err("Only one parameter supported for edit command".to_string()),
                };
            }

            if delete_flag.is_some() {
                return match ctx.params.len() {
                    0 => Err("No note parameter specified".to_string()),
                    _ => delete(ctx),
                };
            }

            if mv_flag.is_some() {
                return match ctx.params.len() {
                    0 => Err("No note parameter specified".to_string()),
                    _ => mv(ctx),
                };
            }

            // It will never get here as only one command flag is specified
            Err("No flag specifed for notes command, but we thought there was one".to_string())
        }
        _ => Err("Cannot specify multiple of new, edit, delete, and move".to_string()),
    }
}

fn list(ctx: &Context) -> Result<()> {
    let list_uuid = cmd::find_list_uuid_or_current(ctx)?;

    let notes = sqlite::notes::all(&ctx.db, &list_uuid)?;

    let mut printer = output::TablePrinter::new(vec!["ID".to_string(), "TITLE".to_string()]);
    for note in notes.iter() {
        printer.add_row(vec![note.id.to_string(), note.title.clone()])?;
    }

    printer.print();
    Ok(())
}

fn show(ctx: &Context) -> Result<()> {
    let list = cmd::find_list_or_current(ctx)?;
    let note = find_note_by_id(ctx, &list.uuid, &ctx.params[0])?;
    output::print(note.id, &note.title, &note.body);
    Ok(())
}

fn create(ctx: &mut Context) -> Result<()> {
    let mut list = cmd::find_list_or_current(ctx)?;
    let title: Option<String>;
    let mut body: Option<String>;

    let text = input::get_file_input(None);
    if let Some(result) = utils::split_text_into_title_desc(&text) {
        let (t, b) = result;
        title = t;
        body = b;
    } else {
        return Err("Failed to get note data".to_string());
    }

    if title.is_none() {
        return Err("No title specified".to_string());
    }

    if body.is_none() {
        body = Some(String::new())
    }

    if let Err(_) = sqlite::transaction(&mut ctx.db, |tx| {
        let now = utils::now();
        let note = models::Note {
            uuid: uuid::Uuid::new_v4(),
            id: list.next_note_id,
            title: title.as_ref().unwrap().clone(),
            body: body.as_ref().unwrap().clone(),
            created: now,
            modified: now,
            list_uuid: list.uuid,
        };
        list.next_note_id += 1;

        sqlite::update_list(tx, &list)?;
        sqlite::notes::create(tx, &note)?;
        sqlite::create_history(
            tx,
            &models::History {
                uuid: uuid::Uuid::new_v4(),
                command: CMD_NOTE_CREATE.to_string(),
                state: utils::encode_history_state(&models::CmdNoteState {
                    uuid: note.uuid,
                    title: note.title.clone(),
                    body: note.body.clone(),
                    created: note.created,
                    modified: note.modified,
                    list_uuid: note.list_uuid,
                })?,
                timestamp: now,
                synced: false,
            },
        )?;

        Ok(())
    }) {
        return Err("Failed to create note".to_string());
    }

    Ok(())
}

fn edit(ctx: &mut Context) -> Result<()> {
    let list = cmd::find_list_or_current(ctx)?;
    let mut note = find_note_by_id(ctx, &list.uuid, &ctx.params[0])?;

    let mut title: Option<String> = None;
    let mut body: Option<String> = None;

    let current = vec![note.title.clone(), String::from(""), note.body.clone()].join("\n");
    let text = input::get_file_input(Some(&current));
    if let Some(result) = utils::split_text_into_title_desc(&text) {
        let (t, b) = result;
        title = t;
        body = b;
    }

    if title.is_some() || body.is_some() {
        let now = utils::now();

        if let Some(t) = title {
            note.title = t;
        }

        if let Some(b) = body {
            note.body = b;
        }

        note.modified = now;

        sqlite::transaction(&mut ctx.db, |tx| {
            sqlite::notes::update(tx, &note)?;

            sqlite::create_history(
                tx,
                &models::History {
                    uuid: uuid::Uuid::new_v4(),
                    command: CMD_NOTE_UPDATE.to_string(),
                    state: utils::encode_history_state(&models::CmdNoteState {
                        uuid: note.uuid,
                        title: note.title.clone(),
                        body: note.body.clone(),
                        created: note.created,
                        modified: note.modified,
                        list_uuid: note.list_uuid,
                    })?,
                    timestamp: now,
                    synced: false,
                },
            )?;
            Ok(())
        })?;
    }

    Ok(())
}

fn delete(ctx: &mut Context) -> Result<()> {
    let list_uuid = cmd::find_list_uuid_or_current(ctx)?;
    for id in ctx.params.iter() {
        let note = find_note_by_id(&ctx, &list_uuid, id)?;

        println!("Are you sure you want to delete note '{}'?", note.title);
        println!("This cannot be undone!");
        print!("Delete (y/N)? ");

        match input::get_stdin_input().to_lowercase().as_str() {
            "y" | "yes" => {
                if let Err(e) = sqlite::transaction(&mut ctx.db, |tx| {
                    sqlite::notes::delete(tx, &note)?;

                    sqlite::create_history(
                        tx,
                        &models::History {
                            uuid: uuid::Uuid::new_v4(),
                            command: CMD_NOTE_DELETE.to_string(),
                            state: utils::encode_history_state(&models::CmdDeleteState {
                                uuid: note.uuid,
                            })?,
                            timestamp: utils::now(),
                            synced: false,
                        },
                    )?;

                    Ok(())
                }) {
                    println!("Skipping {}: Failed to delete: {}", id, e);
                } else {
                    println!("Deleted {}", id);
                }
            }
            _ => {
                println!("Skipping {}", id);
            }
        };
    }

    Ok(())
}

fn mv(ctx: &mut Context) -> Result<()> {
    let list_uuid = cmd::find_list_uuid_or_current(ctx)?;
    let target_id = ctx.data.get("move").unwrap();
    let mut target_list = cmd::find_list_by_id(ctx, target_id)?;
    let now = utils::now();

    for id in ctx.params.iter() {
        let mut note = find_note_by_id(ctx, &list_uuid, id)?;

        println!(
            "Moving note: {} to list: {} with new id: {}",
            note.id, target_id, target_list.next_note_id
        );

        note.id = target_list.next_note_id;
        note.list_uuid = target_list.uuid;
        note.modified = now;
        target_list.next_note_id += 1;

        sqlite::transaction(&mut ctx.db, |tx| {
            sqlite::update_list(tx, &target_list)?;
            sqlite::notes::mv(tx, &note)?;

            sqlite::create_history(
                tx,
                &models::History {
                    uuid: uuid::Uuid::new_v4(),
                    command: CMD_NOTE_MOVE.to_string(),
                    state: utils::encode_history_state(&models::CmdMoveState {
                        uuid: note.uuid,
                        list: target_list.uuid,
                    })?,
                    timestamp: now,
                    synced: false,
                },
            )?;
            Ok(())
        })?;
    }

    Ok(())
}

fn find_note_by_id(ctx: &Context, list_uuid: &uuid::Uuid, id: &String) -> Result<models::Note> {
    match sqlite::notes::get(&ctx.db, list_uuid, id) {
        Ok(res) => Ok(res),
        Err(_) => Err(format!("Failed to find note with id: {}", id)),
    }
}

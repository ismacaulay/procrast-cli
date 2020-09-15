use crate::{
    models, network, sqlite,
    utils::{self, Result},
    Context,
};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Deserialize)]
struct HistoryResponse {
    history: Vec<models::ApiHistory>,
}

pub fn run(ctx: &mut Context) -> Result<()> {
    // get the last local sync time
    // send request to /history with the last local sync time (or empty if none)
    //      for each of the results, add to local db
    //      if there is another page send another request, else finish
    // update last local sync time with now
    let endpoint = if let Some(last_local_sync) = get_last_local_sync(ctx) {
        format!("/history?since={}", last_local_sync)
    } else {
        String::from("/history")
    };

    match network::send_get_request::<HistoryResponse>(&ctx.client, &endpoint) {
        Ok(resp) => {
            sqlite::transaction(&mut ctx.db, |tx| {
                for history in resp.history.iter() {
                    match history.command.as_str() {
                        "LIST CREATE" => handle_list_create(tx, history),
                        "LIST UPDATE" => handle_list_update(tx, history),
                        "LIST DELETE" => handle_list_delete(tx, history),
                        _ => Err(format!("Unknown history command: {}", history.command)),
                    }?;

                    sqlite::create_history(
                        tx,
                        &models::History {
                            uuid: history.uuid,
                            command: history.command.clone(),
                            state: history.state.clone(),
                            created: history.created,
                            synced: true,
                        },
                    )?;
                }

                sqlite::set_last_local_sync(tx, utils::now())?;

                Ok(())
            })?;
        }
        Err(e) => {
            println!("Failed to get history {}", e);
            return Err("Failed to get history".to_string());
        }
    };

    // get the last server sync time
    // gather all commands since last server sync that has not been synced
    //      send post to /sync with the commands
    //      update commands synced flag
    //  update last server sync
    Ok(())
}

fn get_last_local_sync(ctx: &Context) -> Option<i64> {
    match sqlite::get_last_local_sync(&ctx.db) {
        Ok(last) => Some(last),
        Err(_) => None,
    }
}

fn decode_history_state<T: DeserializeOwned>(history: &models::ApiHistory) -> Result<T> {
    match base64::decode(history.state.as_bytes()) {
        Ok(state) => match serde_json::from_slice::<T>(&state) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Failed to decode json: {}", e)),
        },
        Err(e) => Err(format!("Failed to decode state: {}", e)),
    }
}

fn handle_list_create(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let list = decode_history_state::<models::ApiList>(history)?;
    let local_id = sqlite::get_next_list_id(conn)?;

    sqlite::create_list(
        conn,
        &models::List {
            uuid: list.uuid,
            id: local_id,
            title: list.title.clone(),
            description: list.description.clone(),
            created: list.created,
            modified: list.modified,
            next_item_id: 1,
        },
    )?;
    sqlite::set_next_list_id(conn, local_id + 1)?;

    Ok(())
}

fn handle_list_update(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let api_list = decode_history_state::<models::ApiList>(history)?;
    let mut list = sqlite::find_list_by_uuid(conn, &api_list.uuid)?;
    list.title = api_list.title.clone();
    list.description = api_list.description.clone();
    list.modified = api_list.modified;

    sqlite::update_list(conn, &list)?;

    Ok(())
}

fn handle_list_delete(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let api_list = decode_history_state::<models::CmdListDeleteState>(history)?;
    let list = sqlite::find_list_by_uuid(conn, &api_list.uuid)?;

    for item in sqlite::get_items(conn, &list.uuid)?.iter() {
        sqlite::delete_item(conn, item)?;
    }

    sqlite::delete_list(conn, &list)?;

    Ok(())
}

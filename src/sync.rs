use crate::{
    log,
    models::{
        self, CMD_ITEM_CREATE, CMD_ITEM_DELETE, CMD_ITEM_UPDATE, CMD_LIST_CREATE, CMD_LIST_DELETE,
        CMD_LIST_UPDATE,
    },
    network, sqlite,
    utils::{self, Result},
    Context,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct HistoryResponse {
    history: Vec<models::ApiHistory>,
}

#[derive(Debug, Serialize)]
struct HistoryPostRequest {
    history: Vec<models::ApiHistory>,
}

#[derive(Debug, Deserialize)]
struct HistoryPostResponse {
    processed: Vec<uuid::Uuid>,
}

pub fn run(ctx: &mut Context) -> Result<()> {
    if ctx.config.base_url == "" {
        return Err(String::from("No base_url configured"));
    }

    let all = match ctx.data.get("all") {
        Some(_) => true,
        None => false,
    };

    let base_url = format!("{}/procrast/v1", ctx.config.base_url);

    // get the last local sync time
    // send request to /history with the last local sync time (or empty if none)
    //      for each of the results, add to local db
    //      if there is another page send another request, else finish
    // update last local sync time with now
    let endpoint = if let Some(last_local_sync) = get_last_local_sync(ctx) {
        if !all {
            format!("{}/history?since={}", base_url, last_local_sync)
        } else {
            format!("{}/history", base_url)
        }
    } else {
        format!("{}/history", base_url)
    };

    match network::send_get_request::<HistoryResponse>(
        &ctx.client,
        &endpoint,
        Some(&ctx.config.token),
    ) {
        Ok(resp) => {
            sqlite::transaction(&mut ctx.db, |tx| {
                sqlite::set_last_local_sync(tx, utils::now())?;

                for history in resp.history.iter() {
                    if let Ok(_) = sqlite::find_history_by_uuid(tx, &history.uuid) {
                        sqlite::update_history_synced(tx, &history.uuid, true)?;
                        continue;
                    }

                    match history.command.as_str() {
                        CMD_LIST_CREATE => handle_list_create(tx, history),
                        CMD_LIST_UPDATE => handle_list_update(tx, history),
                        CMD_LIST_DELETE => handle_list_delete(tx, history),
                        CMD_ITEM_CREATE => handle_item_create(tx, history),
                        CMD_ITEM_UPDATE => handle_item_update(tx, history),
                        CMD_ITEM_DELETE => handle_item_delete(tx, history),
                        _ => Err(format!("Unknown history command: {}", history.command)),
                    }?;

                    sqlite::create_history(
                        tx,
                        &models::History {
                            uuid: history.uuid,
                            command: history.command.clone(),
                            state: history.state.clone(),
                            timestamp: history.timestamp,
                            synced: true,
                        },
                    )?;
                }

                Ok(())
            })?;
        }
        Err(e) => {
            println!("Failed to get history {}", e);
            return Err("Failed to get history".to_string());
        }
    };

    // gather all commands that has not been synced
    //      send post to /sync with the commands
    //      update commands synced flag
    //  update last server sync
    let mut request = HistoryPostRequest {
        history: Vec::new(),
    };

    let result = if all {
        sqlite::get_history(&ctx.db)?
    } else {
        sqlite::get_unsynced_history(&ctx.db)?
    };
    for history in result.iter() {
        request.history.push(models::ApiHistory {
            uuid: history.uuid,
            command: history.command.clone(),
            state: history.state.clone(),
            timestamp: history.timestamp,
        })
    }

    let url = format!("{}/history", base_url);
    if request.history.len() > 0 {
        match network::send_post_request::<HistoryPostRequest, HistoryPostResponse>(
            &ctx.client,
            &url,
            &request,
            Some(&ctx.config.token),
        ) {
            Ok(resp) => {
                for processed in resp.processed.iter() {
                    if let Err(e) = sqlite::update_history_synced(&ctx.db, &processed, true) {
                        log::println(format!("Failed to updated history synced state: {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to send post request: {}", e));
            }
        };
    }

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
    let list = decode_history_state::<models::CmdListState>(history)?;
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
    let api_list = decode_history_state::<models::CmdListState>(history)?;
    let mut list = sqlite::find_list_by_uuid(conn, &api_list.uuid)?;
    list.title = api_list.title.clone();
    list.description = api_list.description.clone();
    list.modified = api_list.modified;

    sqlite::update_list(conn, &list)?;

    Ok(())
}

fn handle_list_delete(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let api_list = decode_history_state::<models::CmdDeleteState>(history)?;
    if let Ok(list) = sqlite::find_list_by_uuid(conn, &api_list.uuid) {
        for item in sqlite::get_items(conn, &list.uuid)?.iter() {
            sqlite::delete_item(conn, item)?;
        }

        sqlite::delete_list(conn, &list)?;
    }

    Ok(())
}

fn handle_item_create(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let state = decode_history_state::<models::CmdItemState>(history)?;

    let mut list = sqlite::find_list_by_uuid(conn, &state.list_uuid)?;

    sqlite::create_item(
        conn,
        &models::Item {
            uuid: state.uuid,
            id: list.next_item_id,
            title: state.title.clone(),
            description: state.description.clone(),
            state: state.state,
            created: state.created,
            modified: state.modified,
            list_uuid: state.list_uuid,
        },
    )?;

    list.next_item_id += 1;
    sqlite::update_list(conn, &list)?;

    Ok(())
}

fn handle_item_update(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let state = decode_history_state::<models::CmdItemState>(history)?;
    let mut item = sqlite::find_item_by_uuid(conn, &state.uuid)?;
    item.title = state.title.clone();
    item.description = state.description.clone();
    item.state = state.state;
    item.modified = state.modified;

    sqlite::update_item(conn, &item)?;

    Ok(())
}

fn handle_item_delete(conn: &rusqlite::Connection, history: &models::ApiHistory) -> Result<()> {
    let state = decode_history_state::<models::CmdDeleteState>(history)?;
    if let Ok(item) = sqlite::find_item_by_uuid(conn, &state.uuid) {
        sqlite::delete_item(conn, &item)?;
    }

    Ok(())
}

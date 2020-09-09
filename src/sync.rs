use crate::Context;

fn sync(ctx: &Context) {
    // if there is no ts sync all local data, otherwise sync data since ts
    // match ctx.db.get_last_sync() {
    //     Some(ts) => {
    //         // sync data since ts
    //     }
    //     None => {
    //         // sync all data
    //     }
    // }
}

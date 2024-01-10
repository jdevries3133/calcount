use crate::prelude::*;

#[derive(Deserialize)]
pub struct Checkpoint {
    date: NaiveDate,
}

impl Component for Checkpoint {
    fn render(&self) -> String {
        let date = self.date;
        let date_str = self.date.format("%m/%d/%Y");
        let delete = Route::BalancingDeleteCheckpoint;
        format!(
            r#"
            <form hx-delete="{delete}">
                <p class="text-lg font-semibold my-1">
                    {date_str}
                        <input type="hidden" value="{date}" name="date" />
                        <button
                            class="p-1 bg-red-100 hover:bg-red-200 rounded text-sm text-black"
                        >
                            Delete
                        </button>
                </p>
            </form>
            "#
        )
    }
}

struct CheckpointList<'a> {
    prev_checkpoints: &'a [Checkpoint],
}
impl Component for CheckpointList<'_> {
    fn render(&self) -> String {
        let create_checkpoint = Route::BalancingCreateCheckpoint;
        let checkpoints =
            self.prev_checkpoints
                .iter()
                .fold(String::new(), |mut acc, ch| {
                    acc.push_str(&ch.render());
                    acc
                });
        format!(
            r##"
            <div class="prose dark:text-slate-200">
                <h1 class="dark:text-slate-200 mb-2">Balancing Checkpoints</h1>
                <details class="mb-2">
                    <summary>What are balancing checkpoints?</summary>
                    <p>
                        All meals before a checkpoint will be ignored for the
                        purpose of calorie balancing. Most of the time, calorie
                        balancing ensures that excess or defecit calories from
                        previous days will rollover into our goals for future
                        days. If we eat too much, our future goal will decrease.
                        If we eat too little, our future goal increases.
                    </p>
                    <p>
                        Of course, this system is all fine and good until life
                        happens and you miss counting a few meals! In this case,
                        you can set a checkpoint, and Bean Count will ignore all
                        meals before the checkpoint date. This allows you to give
                        yourself a clean slate as-of any date at any time as many
                        times as you'd like.
                    </p>
                </details>
            </div>
            <h2 class="dark:text-slate-200 text-lg font-semibold">
                Create Checkpoint
            </h2>
            <form
                hx-post="{create_checkpoint}"
                hx-target="#prev-checkpoint-list"
                hx-swap="afterbegin"
            >
                <label for="date">New Checkpoint Date</label>
                <input id="date" type="date" name="date" />
                <button
                    class="block bg-blue-100 hover:bg-blue-200 rounded p-2 my-1"
                >
                    Save
                </button>
            </form>
            <h2 class="dark:text-slate-200 text-lg font-semibold">
                Previous Checkpoints
            </h2>
            <div id="prev-checkpoint-list">
                {checkpoints}
            </div>
            "##
        )
    }
}

pub async fn checkpoint_list(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "checkpoint list")?;
    let checkpoints = query_as!(
        Checkpoint,
        "select ignore_before as date
        from balancing_checkpoint where user_id = $1
        order by date desc",
        session.user_id
    )
    .fetch_all(&db)
    .await?;
    Ok(Page {
        title: "Checkpoint List",
        children: &PageContainer {
            children: &CheckpointList {
                prev_checkpoints: &checkpoints,
            },
        },
    }
    .render())
}

pub async fn create_checkpoint(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(checkpoint): Form<Checkpoint>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "create checkpoint")?;
    let res = query!(
        "insert into balancing_checkpoint (user_id, ignore_before)
        values ($1, $2)
        on conflict do nothing
        ",
        session.user_id,
        checkpoint.date
    )
    .execute(&db)
    .await?;

    if res.rows_affected() == 0 {
        // This is a little edge-case where we click the save button twice.
        // Our "on conflict do nothing" does nothing, but then we re-render
        // the checkpoint again, which appends it into the list on the client.
        // If we didn't actually create a new checkpoint, we don't want to
        // insert anything into the checkpoint list either.
        Ok("".into())
    } else {
        Ok(checkpoint.render())
    }
}

pub async fn delete_checkpoint(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(checkpoint): Form<Checkpoint>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "delete checkpoint")?;
    query!(
        "delete from balancing_checkpoint
        where ignore_before = $1 and user_id = $2",
        checkpoint.date,
        session.user_id
    )
    .execute(&db)
    .await?;
    Ok("")
}

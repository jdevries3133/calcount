use crate::prelude::*;

#[derive(Deserialize)]
pub struct Checkpoint {
    pub date: NaiveDate,
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
        let home = Route::UserHome;
        let checkpoints = if self.prev_checkpoints.is_empty() {
            "No checkpoints added yet!".into()
        } else {
            self.prev_checkpoints
                .iter()
                .fold(String::new(), |mut acc, ch| {
                    acc.push_str(&ch.render());
                    acc
                })
        };
        format!(
            r##"
            <div class="prose dark:text-slate-200">
                <button
                    class="dark:bg-emerald-700 dark:hover:bg-emerald-800
                    bg-emerald-100 hover:bg-emerald-200 p-1 m-1 rounded"
                    onclick="history.back()"
                >
                    Back
                </button>
                <a href="{home}">
                    <button
                        class="dark:bg-emerald-700 dark:hover:bg-emerald-800
                        bg-emerald-100 hover:bg-emerald-200 p-1 m-1 rounded
                        dark:text-slate-200"
                    >
                        Home
                    </button>
                </a>
                <h1 class="dark:text-slate-200 mb-2">Balancing Checkpoints</h1>
                <details class="mb-2">
                    <summary>What are balancing checkpoints?</summary>
                    <p>
                        All meals before a checkpoint will be ignored for the
                        purpose of calorie balancing. Most of the time, calorie
                        balancing ensures that excess or defecit calories from
                        previous days will rollover into your goal for today.
                        If you ate too much yesterday, today's goal will be a
                        bit lower, and visa versa.
                    </p>
                    <p>
                        Of course, this system breaks down if you skip calorie
                        counting for a few days! If that happens, set a
                        checkpoint to "reset" calorie balancing as-of a date
                        of your choice. Checkpoints give you a clean slate.
                    <p>
                        Note that if you set a checkpoint date in the future,
                        you effectively disable calorie balancing until that
                        date arrives, which might be handy feature if you
                        have an upcoming holiday or vacation where you're
                        planning to pause calorie counting!
                    </p>
                </details>
            </div>
            <div class="bg-emerald-200 dark:bg-indigo-900 rounded p-2 my-2">
                <h2 
                    class="text-lg font-semibold
                    rounded-xl mt-4 mb-2 inline-block"
                >
                    Create Checkpoint
                </h2>
                <form
                    hx-post="{create_checkpoint}"
                    hx-target="#prev-checkpoint-list"
                    hx-swap="afterbegin"
                >
                    <label class="block" for="date">Date</label>
                    <input id="date" type="date" name="date" />
                    <button
                        class="block rounded p-2 my-1 dark:bg-indigo-500
                        dark:hover:bg-indigo-600 text-black dark:text-white
                        bg-emerald-100 hover:bg-emerald-300 font-semibold"
                    >
                        Save
                    </button>
                </form>
            </div>
            <div class="bg-emerald-200 dark:bg-indigo-900 rounded p-2 my-2">
                <h2
                    class="text-lg font-semibold
                    rounded-xl mt-4 mb-2 inline-block"
                >
                    Previous Checkpoints
                </h2>
                <div id="prev-checkpoint-list">
                    {checkpoints}
                </div>
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

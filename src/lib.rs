// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use std::collections::{BTreeMap, VecDeque};

use chrono::{NaiveDate, NaiveDateTime};
use seed::{prelude::*, *};
use uuid::Uuid;
// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        current_date: chrono::offset::Local::now().date().naive_local(),
        planned_work_periods: VecDeque::new(),
        work_sleep_data: BTreeMap::new(),
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    current_date: NaiveDate,
    planned_work_periods: VecDeque<Period>,
    work_sleep_data: BTreeMap<NaiveDate, WorkSleep>,
}

struct Period {
    id: Uuid,
    name: String,
}

struct WorkSleep {
    target_work_count: i64,
    actual_work_count: i64,
    target_sleep_time: NaiveDateTime,
    actual_sleep_time: NaiveDateTime,
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Copy, Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    Increment,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.current_date = model.current_date.succ(),
    }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    div![
        "This is a counter: ",
        C!["counter"],
        button![
            model.current_date.to_string(),
            ev(Ev::Click, |_| Msg::Increment),
        ],
    ]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}

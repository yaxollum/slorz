// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use std::collections::{BTreeMap, VecDeque};

use chrono::{NaiveDate, NaiveDateTime};
use seed::{prelude::*, *};
use uuid::Uuid;
use web_sys::HtmlInputElement;
// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        data: Data {
            current_date: chrono::offset::Local::now().date().naive_local(),
            new_task: NewTask {
                name: String::new(),
                quantity: "1".to_owned(),
            },
            planned_work_periods: VecDeque::new(),
            work_sleep_data: BTreeMap::new(),
        },
        refs: Refs::default(),
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.

struct Model {
    data: Data,
    refs: Refs,
}

struct Data {
    current_date: NaiveDate,
    new_task: NewTask,
    planned_work_periods: VecDeque<Period>,
    work_sleep_data: BTreeMap<NaiveDate, WorkSleep>,
}

#[derive(Default)]
struct Refs {}

#[derive(Debug)]
struct NewTask {
    name: String,
    quantity: String,
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
#[derive(Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    Increment,
    AddNewTask,
    NewTaskNameChanged(String),
    NewTaskQuantityChanged(String),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.data.current_date = model.data.current_date.succ(),
        Msg::AddNewTask => {
            let quantity: i64 = model.data.new_task.quantity.parse().unwrap_or(1);
            for _ in 0..quantity {
                let period = Period {
                    id: Uuid::new_v4(),
                    name: model.data.new_task.name.clone(),
                };
                model.data.planned_work_periods.push_back(period);
            }
        }
        Msg::NewTaskNameChanged(s) => {
            model.data.new_task.name = s;
        }
        Msg::NewTaskQuantityChanged(s) => {
            model.data.new_task.quantity = s;
        }
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
            model.data.current_date.to_string(),
            ev(Ev::Click, |_| Msg::Increment),
        ],
        ul![model
            .data
            .planned_work_periods
            .iter()
            .map(|wp| li![&wp.name])],
        input![
            attrs! {At::Placeholder=>"Name of task"},
            input_ev(Ev::Input, Msg::NewTaskNameChanged)
        ],
        raw!["&times;"],
        input![
            attrs! {At::Placeholder=>"Quantity",At::Value=>model.data.new_task.quantity},
            input_ev(Ev::Input, |quantity| {
                Msg::NewTaskQuantityChanged(quantity)
            })
        ],
        input![attrs![
            At::Type => "range",
            At::Min => "100",
            At::Max => "800",
            At::Step => "50",
            At::Value => "400",
        ]],
        button!["Add new task", ev(Ev::Click, |_| Msg::AddNewTask)]
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

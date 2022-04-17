// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use std::collections::{BTreeMap, VecDeque};

use chrono::{Duration, NaiveDate, NaiveTime};
use seed::{prelude::*, *};
use uuid::Uuid;
use web_sys::HtmlInputElement;
// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let current_date = chrono::offset::Local::now().date().naive_local();
    Model {
        data: Data {
            current_date,
            new_task: NewTask {
                name: String::new(),
                quantity: "1".to_owned(),
            },
            planned_work_periods: VecDeque::new(),
            default_work_sleep_goals: WorkSleepGoals {
                work_sleep_balance: 70,
                target_work_count: 6,
                target_bedtime: Bedtime {
                    time: NaiveTime::from_hms(11, 0, 0),
                    next_day: false,
                },
                bedtime_pts_halflife: 30,
            },
            work_sleep_data: WorkSleepData::new(current_date - Duration::days(6)),
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
    default_work_sleep_goals: WorkSleepGoals,
    work_sleep_data: WorkSleepData,
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

#[derive(Debug)]
struct WorkSleepData {
    week_start: NaiveDate,
    data: BTreeMap<NaiveDate, WorkSleep>,
}
impl WorkSleepData {
    fn new(week_start: NaiveDate) -> Self {
        Self {
            week_start,
            data: BTreeMap::new(),
        }
    }
    fn get_mut_or_create(
        &mut self,
        date: &NaiveDate,
        work_sleep_goals: &WorkSleepGoals,
    ) -> &mut WorkSleep {
        self.data.entry(date.clone()).or_insert(WorkSleep {
            goals: work_sleep_goals.clone(),
            actual_work_count: 0,
            actual_bedtime: None,
        })
    }
    fn get_current_week(&self) -> Vec<(NaiveDate, Option<&WorkSleep>)> {
        let mut relevant = Vec::new();
        let mut current = self.week_start;

        for _ in 0..7 {
            relevant.push((current, self.data.get(&current)));
            current = current.succ();
        }
        relevant
    }
    fn set_week_start(&mut self, current_date: &NaiveDate) {
        let is_latest = if let Some((last_date, _)) = self.data.iter().rev().next() {
            last_date <= current_date
        } else {
            true
        };
        self.week_start = if is_latest {
            *current_date - Duration::days(6)
        } else {
            *current_date - Duration::days(3)
        };
    }
}

#[derive(Clone, Debug)]
struct Bedtime {
    time: NaiveTime,
    next_day: bool,
}

impl Bedtime {
    fn abs_diff(&self, other: &Self) -> i64 {
        let one_day = Duration::days(1);
        let zero = Duration::zero();
        (other.time - self.time + if other.next_day { one_day } else { zero }
            - if self.next_day { one_day } else { zero })
        .num_minutes()
        .abs()
    }
}

#[derive(Clone, Debug)]
struct WorkSleepGoals {
    work_sleep_balance: i64,
    target_work_count: i64,
    target_bedtime: Bedtime,
    bedtime_pts_halflife: i64,
}

#[derive(Debug)]
struct WorkSleep {
    goals: WorkSleepGoals,
    actual_work_count: i64,
    actual_bedtime: Option<Bedtime>,
}

impl WorkSleep {
    fn calc_score(&self) -> i64 {
        let work_score = (self.actual_work_count * self.goals.work_sleep_balance) as f64
            / self.goals.target_work_count as f64;
        let sleep_score = if let Some(actual_bedtime) = &self.actual_bedtime {
            (100 - self.goals.work_sleep_balance) as f64
                * (0.5f64).powf(
                    actual_bedtime.abs_diff(&self.goals.target_bedtime) as f64
                        / (self.goals.bedtime_pts_halflife as f64),
                )
        } else {
            0.0
        };
        (work_score + sleep_score).round() as i64
    }
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    SetCurrentDate(NaiveDate),
    AddNewTask,
    DeleteTask(Uuid),
    MoveTaskToTop(Uuid),
    MoveTaskUp(Uuid),
    FinishedTopTask,
    NewTaskNameChanged(String),
    NewTaskQuantityChanged(String),
    ViewNextWeek,
    ViewPreviousWeek,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetCurrentDate(date) => {
            model.data.current_date = date;
            model.data.work_sleep_data.set_week_start(&date);
        }
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
        Msg::DeleteTask(id) => {
            model.data.planned_work_periods.retain(|wp| wp.id != id);
        }
        Msg::MoveTaskToTop(id) => {
            let i = model
                .data
                .planned_work_periods
                .iter()
                .position(|wp| wp.id == id);
            if let Some(i) = i {
                let wp = model.data.planned_work_periods.remove(i).unwrap();
                model.data.planned_work_periods.push_front(wp);
            }
        }
        Msg::MoveTaskUp(id) => {
            let i = model
                .data
                .planned_work_periods
                .iter()
                .position(|wp| wp.id == id);
            if let Some(i) = i {
                if let Some(j) = i.checked_sub(1) {
                    model.data.planned_work_periods.swap(i, j)
                }
            }
        }
        Msg::FinishedTopTask => {
            model.data.planned_work_periods.pop_front();
            model
                .data
                .work_sleep_data
                .get_mut_or_create(
                    &model.data.current_date,
                    &model.data.default_work_sleep_goals,
                )
                .actual_work_count += 1;
            log!(model.data.work_sleep_data);
        }
        Msg::NewTaskNameChanged(s) => {
            model.data.new_task.name = s;
        }
        Msg::NewTaskQuantityChanged(s) => {
            model.data.new_task.quantity = s;
        }
        Msg::ViewNextWeek => {
            model.data.work_sleep_data.week_start += Duration::weeks(1);
        }
        Msg::ViewPreviousWeek => {
            model.data.work_sleep_data.week_start -= Duration::weeks(1);
        }
    }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.

fn view(model: &Model) -> Node<Msg> {
    div![view_work_sleep_data(model), view_current_date(model)]
}

fn view_work_sleep_data(model: &Model) -> Node<Msg> {
    div![
        button!["Previous Week", ev(Ev::Click, |_| Msg::ViewPreviousWeek),],
        button!["Next Week", ev(Ev::Click, |_| Msg::ViewNextWeek),],
        table![tr![model
            .data
            .work_sleep_data
            .get_current_week()
            .iter()
            .map(|(date, ws)| view_work_sleep_data_one_day(
                *date,
                ws,
                *date == model.data.current_date
            ))]]
    ]
}
fn view_current_date(model: &Model) -> Node<Msg> {
    div![
        ul![
            if let Some(wp) = model.data.planned_work_periods.front() {
                Some(view_first_work_period(&wp.name, wp.id))
            } else {
                None
            },
            model
                .data
                .planned_work_periods
                .iter()
                .skip(1)
                .map(|wp| view_work_period(&wp.name, wp.id)),
        ],
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

fn view_work_sleep_data_one_day(
    date: NaiveDate,
    ws: &Option<&WorkSleep>,
    is_current_date: bool,
) -> Node<Msg> {
    td![
        IF!(is_current_date=>vec![span!["CURRENT DATE"],br![]]),
        button![
            date.to_string(),
            ev(Ev::Click, move |_| Msg::SetCurrentDate(date)),
        ],
        br![],
        if let Some(ws) = ws {
            div![
                span![format!("Work Completed: {}", ws.actual_work_count)],
                br![],
                span![if let Some(actual_bedtime) = &ws.actual_bedtime {
                    format!("Bedtime: {}", actual_bedtime.time.to_string())
                } else {
                    "No bedtime data".to_owned()
                }],
                br![],
                span![format!("Score: {}", ws.calc_score())],
            ]
        } else {
            span!["No data"]
        }
    ]
}
fn view_first_work_period(name: &str, id: Uuid) -> Node<Msg> {
    li![div![
        label![format!("CURRENT TASK: {}", name)],
        button!["Delete", ev(Ev::Click, move |_| Msg::DeleteTask(id))],
        button!["DONE!", ev(Ev::Click, |_| Msg::FinishedTopTask)],
    ]]
}
fn view_work_period(name: &str, id: Uuid) -> Node<Msg> {
    li![div![
        label![name],
        button!["Delete", ev(Ev::Click, move |_| Msg::DeleteTask(id))],
        button![
            "Move to top",
            ev(Ev::Click, move |_| Msg::MoveTaskToTop(id))
        ],
        button!["Move up", ev(Ev::Click, move |_| Msg::MoveTaskUp(id))],
    ]]
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

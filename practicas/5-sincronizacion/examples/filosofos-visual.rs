extern crate druid;
extern crate std_semaphore;
extern crate rand;

use druid::{AppLauncher, Data, ImageBuf, PlatformError, Widget, WidgetExt, WindowDesc, ExtEventSink, Selector, Target, AppDelegate, DelegateCtx, Command, Env, Handled};
use druid::widget::{Button, FillStrat, Flex, Image, Label};
use std_semaphore::Semaphore;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::sync::atomic::{AtomicI32, Ordering};
use std::error::Error;
use std::borrow::Borrow;

#[derive(Clone, Data, PartialEq, Debug)] // Debug to easily convert to string
enum PhilosopherState {
    Thinking,
    AwaitingFirstChopstick,
    GotFirstChopstick,
    AwaitingSecondChopstick,
    Eating,
}

struct PhilosopherStateUpdate {
    id: usize,
    state: PhilosopherState,
}

const SET_PHILOSOPHER_STATE: Selector<PhilosopherStateUpdate> = Selector::new("set-philosopher-state");

#[derive(Clone, Data)]
struct AppState {
    states: [PhilosopherState; N],
    clicks: Arc<Vec<Semaphore>>,
}

const images: [&[u8]; 5] = [
    include_bytes!("../img/philo0.png"),
    include_bytes!("../img/philo1.png"),
    include_bytes!("../img/philo2.png"),
    include_bytes!("../img/philo3.png"),
    include_bytes!("../img/philo4.png")
];

const N: usize = 5;

fn main() -> Result<(), PlatformError> {
    // Window builder. We set title and size
    let main_window = WindowDesc::new(ui_builder)
        .title("Filosofos")
        .window_size((800.0, 800.0));

    let launcher = AppLauncher::with_window(main_window).delegate(Delegate {});

    let chopsticks: Arc<Vec<Semaphore>> = Arc::new((0..N)
        .map(|_| Semaphore::new(1))
        .collect());

    let app_state = AppState {
        states: [PhilosopherState::Thinking, PhilosopherState::Thinking, PhilosopherState::Thinking, PhilosopherState::Thinking, PhilosopherState::Thinking],
        clicks: Arc::new((0..N)
            .map(|_| Semaphore::new(0))
            .collect()),
    };

    for id in (0..N) {
        let chopsticks_local = chopsticks.clone();
        let clicks = app_state.clicks.clone();
        let sink = launcher.get_external_handle();
        thread::spawn(move || philosopher(id, chopsticks_local, &clicks[id], sink));
    }

    // Run the app
    launcher.launch(app_state)
}

fn philosopher(id: usize, chopsticks: Arc<Vec<Semaphore>>, await_click: &Semaphore, event_sink: ExtEventSink) {
    let first_chopstick = &chopsticks[id];
    let second_chopstick = &chopsticks[(id + 1) % 5];

    let notify_state = |state: PhilosopherState| event_sink.submit_command(SET_PHILOSOPHER_STATE, PhilosopherStateUpdate { id, state }, Target::Auto);

    loop {
        notify_state(PhilosopherState::Thinking);
        await_click.acquire();

        {
            notify_state(PhilosopherState::AwaitingFirstChopstick);
            let first = first_chopstick.access();
            notify_state(PhilosopherState::GotFirstChopstick);

            // Require another click to be able to test the deadlock
            await_click.acquire();

            {
                notify_state(PhilosopherState::AwaitingSecondChopstick);
                let second = second_chopstick.acquire();

                notify_state(PhilosopherState::Eating);
                await_click.acquire();
            }
        }
    }
}

fn ui_builder() -> impl Widget<AppState> {
    let mut flex = Flex::column();

    flex.add_child(render_philosopher(0));
    for i in 1..(N/2 + 1) {
        flex.add_child(
            Flex::row()
                .with_child(render_philosopher(N - i))
                .with_spacer(300.0)
                .with_child(render_philosopher(i))
        );
        flex.add_spacer(20.0);
    }


    flex
}

fn render_philosopher(i: usize) -> impl Widget<AppState> {
    Flex::column()
        .with_child(Image::new(ImageBuf::from_data(images[i % N]).unwrap())
            .fill_mode(FillStrat::ScaleDown)
            .fix_height(150.0))
        .with_child(Label::new(move |data: &AppState, _: &_| i.to_string() + " is " + &*format!("{:?}", data.states[i])))
        .with_spacer(1.0)
        .with_child(Button::new("step")
            .on_click(move |_ctx, data: &mut AppState, _env| data.clicks[i].release())
            .padding(5.0))
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(&mut self, _ctx: &mut DelegateCtx, _target: Target, cmd: &Command, data: &mut AppState, _env: &Env) -> Handled {
        if let Some(update) = cmd.get(SET_PHILOSOPHER_STATE) {
            data.states[update.id] = update.state.clone();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

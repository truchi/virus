use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::Key,
    window::{Window, WindowId},
};

fn main() {
    let (actor_sender, actor_receiver) = unbounded_channel();
    let state = Arc::new(Mutex::new(State {
        actor_sender,
        foo: 0,
    }));

    std::thread::spawn({
        let state = state.clone();

        || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    tokio::spawn(async {
                        Actor {
                            state,
                            actor_receiver,
                        }
                        .run()
                        .await;
                    })
                    .await
                    .unwrap();
                });
        }
    });

    App::run(state);
}

struct App {
    state: Arc<Mutex<State>>,
    _window: Window,
}

impl App {
    pub fn run(state: Arc<Mutex<State>>) {
        let event_loop = EventLoop::new().expect("Cannot create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop
            .run_app(&mut Handler::Uninitialized(state))
            .unwrap();
    }
}

enum Handler {
    Uninitialized(Arc<Mutex<State>>),
    Initialized(App),
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let state = match self {
            Handler::Uninitialized(state) => state,
            Handler::Initialized(_) => panic!("Already initialized"),
        };

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("LOL")
                    .with_decorations(false),
            )
            .expect("Cannot create window");
        window.set_cursor_visible(false);

        *self = Handler::Initialized(App {
            state: state.clone(),
            _window: window,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = match self {
            Handler::Uninitialized(_) => panic!("Not initialized"),
            Handler::Initialized(app) => app,
        };

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.logical_key.as_ref() {
                        Key::Character(str) if str == "q" => event_loop.exit(),
                        Key::Character(str) if str == "a" => {
                            let state = app.state.lock().unwrap();
                            state
                                .actor_sender
                                .send(ActorMessage::Wait(
                                    Duration::from_secs(1),
                                    Callback(Box::new(|state| {
                                        println!("YES {}", state.foo);
                                        state.foo += 1;
                                    })),
                                ))
                                .unwrap();
                        }
                        _ => (),
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
struct State {
    actor_sender: UnboundedSender<ActorMessage>,
    foo: usize,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

struct Callback(Box<dyn Fn(&mut State) + Send>);

impl std::fmt::Debug for Callback {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
enum ActorMessage {
    Wait(Duration, Callback),
}

struct Actor {
    state: Arc<Mutex<State>>,
    actor_receiver: UnboundedReceiver<ActorMessage>,
}

impl Actor {
    async fn run(mut self) {
        while let Some(message) = self.actor_receiver.recv().await {
            match dbg!(message) {
                ActorMessage::Wait(duration, callback) => self.wait(duration, callback),
            }
        }
    }

    fn wait(&self, duration: Duration, callback: Callback) {
        tokio::spawn({
            let state = self.state.clone();

            async move {
                tokio::time::sleep(duration).await;
                let mut state = state.lock().unwrap();
                callback.0(&mut state);
            }
        });
    }
}

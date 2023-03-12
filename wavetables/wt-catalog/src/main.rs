mod my_canvas;
mod saved_state;

use iced::{
    widget::{self, text, Canvas, Column, Container, Scrollable},
    window, Alignment, Application, Command, Element, Length, Renderer, Settings, Theme,
};
use my_canvas::MyCanvas;

pub fn main() -> iced::Result {
    App::run(Settings {
        window: window::Settings {
            size: (600, 400),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug)]
enum App {
    Main,
}

#[derive(Debug, Default)]
struct State {}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<saved_state::SavedState, saved_state::LoadError>),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App::Main,
            // Command::perform(SavedState::load(), Message::Loaded),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("wavetable catalog")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Loaded(Ok(state)) => {
                // self.state = state;
            }
            Message::Loaded(Err(error)) => {
                eprintln!("Failed to load saved state: {:?}", error);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        fn make_canvas<'a>(f: impl Fn(f64) -> f64 + 'static) -> Element<'a, Message, Renderer> {
            Container::new(
                Canvas::new(MyCanvas::new(Box::new(f)))
                    .width(64.0)
                    .height(64.0),
            )
            .padding(8.0)
            .into()
        }
        let mut column = widget::Column::new();
        column = column.push({
            let mut row = widget::Row::new().push(text("primitive"));

            for f in [
                wavetables::primitives::sin,
                wavetables::primitives::saw,
                wavetables::primitives::triangle,
                wavetables::primitives::shifted_triangle,
                wavetables::primitives::square,
                wavetables::primitives::quadratic,
            ] {
                row = row.push(make_canvas(f));
            }
            row
        });
        column = column.push({
            let mut row = widget::Row::new().push(text("bend"));

            row = row.push(make_canvas(|t| {
                wavetables::primitives::saw(wavetables::bend::quadratic_bender(1.0)(t))
            }));
            row = row.push(make_canvas(|t| {
                wavetables::primitives::saw(wavetables::bend::quadratic_bender(-1.0)(t))
            }));
            row = row.push(make_canvas(|t| {
                wavetables::primitives::saw(wavetables::bend::cubic_bender(-0.5)(t))
            }));
            row = row.push(make_canvas(|t| {
                wavetables::primitives::saw(wavetables::bend::cubic_bender(1.0)(t))
            }));
            row = row.push(make_canvas(|t| {
                wavetables::primitives::saw(wavetables::bend::cos_bender(1.0)(t))
            }));
            row = row.push(make_canvas(|t| {
                wavetables::primitives::saw(wavetables::bend::sin_bender(1.0)(t))
            }));
            row
        });
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
        for _ in 0..10 {
            column = column.push({
                let mut row = widget::Row::new();

                for _ in 0..10 {
                    row = row.push(make_canvas(
                        rand_wt::Config { least_depth: 2 }
                            .generate(&mut rng)
                            .build(),
                    ));
                }
                row
            });
        }
        Scrollable::new(column)
            // .horizontal_scroll(Default::default())
            .into()
    }
}

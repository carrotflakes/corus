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
struct App {
    mode: Mode,
    count: u32,
    wts: Vec<wavetables::tree::Tree>,
    slots: [wavetables::tree::Tree; 3],
    current_slot: usize,
}

#[derive(Debug, Clone)]
enum Mode {
    Catalog,
    Composite,
}

#[derive(Debug, Default)]
struct State {}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<saved_state::SavedState, saved_state::LoadError>),
    Pressed,
    SetMode(Mode),
    SelectSlot(usize),
    SetSlot(usize, wavetables::tree::Tree),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App { mode: Mode::Catalog, count: 0, wts : wavetables::unique_negative_reverse_primitives::unique_negative_reverse_primitives().into_iter().map(|x| x.instant_params(&[0.25])).collect(), slots: [wavetables::tree::Tree::Sin, wavetables::tree::Tree::Sin, wavetables::tree::Tree::Sin], current_slot: 0},
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
            Message::Pressed => self.count += 1,
            Message::SetMode(mode) => self.mode = mode,
            Message::SelectSlot(slot) => self.current_slot = slot,
            Message::SetSlot(slot, wt) => self.slots[slot] = wt,
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        fn make_canvas<'a>(f: impl Fn(f64) -> f64 + 'static) -> Element<'a, Message, Renderer> {
            Container::new(
                widget::button(
                    Canvas::new(MyCanvas::new(Box::new(f)))
                        .width(64.0)
                        .height(64.0),
                ),
            )
            .padding(8.0)
            .into()
        }
        fn make_canvas_b<'a>(f: impl Fn(f64) -> f64 + 'static, mes: Message) -> Element<'a, Message, Renderer> {
            Container::new(
                widget::button(
                    Canvas::new(MyCanvas::new(Box::new(f)))
                        .width(64.0)
                        .height(64.0),
                )
                .on_press(mes),
            )
            .padding(8.0)
            .into()
        }
        let mut column = widget::Column::new();
        column = column.push(widget::Row::new().push(widget::button(text("catalog")).on_press(Message::SetMode(Mode::Catalog))).push(widget::button(text("composite")).on_press(Message::SetMode(Mode::Composite))));
        match self.mode {
            Mode::Catalog => {

                column = column.push({
                    let mut row = widget::Row::new().push(text("primitive"));
        
                    for f in [
                        wavetables::primitives::sin,
                        wavetables::primitives::saw,
                        wavetables::primitives::shifted_saw,
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
                    let mut row = widget::Row::new().push(text("UNRP"));
        
                    for f in
                        wavetables::unique_negative_reverse_primitives::unique_negative_reverse_primitives()
                    {
                        row = row.push(make_canvas(f.instant_params(&[0.25]).build()));
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
        
                column = column.push({
                    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
                    let tree = rand_wt::Config {
                        least_depth: 1,
                        variable_num: 1,
                    }
                    .generate(&mut rng);
        
                    let mut row = widget::Row::new().push(text("paramaterized"));
        
                    row = row.push(make_canvas(tree.instant_params(&[0.0]).build()));
                    row = row.push(make_canvas(tree.instant_params(&[0.1]).build()));
                    row = row.push(make_canvas(tree.instant_params(&[0.2]).build()));
                    row = row.push(make_canvas(tree.instant_params(&[0.3]).build()));
                    row = row.push(make_canvas(tree.instant_params(&[0.5]).build()));
                    row = row.push(make_canvas(tree.instant_params(&[1.0]).build()));
                    row
                });
        
                let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
                for _ in 0..10 {
                    column = column.push({
                        let mut row = widget::Row::new();
        
                        for _ in 0..10 {
                            row = row.push(make_canvas(
                                rand_wt::Config {
                                    least_depth: 2,
                                    variable_num: 0,
                                }
                                .generate(&mut rng)
                                .build(),
                            ));
                        }
                        row
                    });
                }
            },
            Mode::Composite => {
                column = column.push({
                    let mut row = widget::Row::new().push(text("my WT"));
        
                    for f in
                        self.wts.iter()
                    {
                        row = row.push(make_canvas_b(f.build(), Message::SetSlot(self.current_slot, f.clone())));
                    }
                    row
                });
                column = column.push({
                    let mut row = widget::Row::new().push(text("slots"));
        
                    for (i, f) in
                        self.slots.iter().enumerate()
                    {
                        row = row.push(make_canvas_b(f.build(), Message::SelectSlot(i)));
                    }
                    row
                });
                column = column.push({
                    let mut row = widget::Row::new().push(text("composite"));
        
                    for f in &[
                        wavetables::tree::Tree::Negative(Box::new(self.slots[0].clone())),
                        wavetables::tree::Tree::Reversed(Box::new(self.slots[0].clone())),
                        wavetables::tree::Tree::Join(Box::new(self.slots[0].clone()), Box::new(self.slots[1].clone())),
                        wavetables::tree::Tree::Shift(wavetables::tree::Value::Constant(0.5), Box::new(self.slots[0].clone())),
                        wavetables::tree::Tree::Scale(wavetables::tree::Value::Constant(0.5), Box::new(self.slots[0].clone())),
                        wavetables::tree::Tree::Scale(wavetables::tree::Value::Constant(2.0), Box::new(self.slots[0].clone())),
                        wavetables::tree::Tree::Blend(wavetables::tree::Value::Constant(0.5), Box::new(self.slots[0].clone()), Box::new(self.slots[1].clone())),
                        wavetables::tree::Tree::DynamicBlend(Box::new(self.slots[0].clone()), Box::new(self.slots[1].clone()), Box::new(self.slots[2].clone())),
                        wavetables::tree::Tree::Product(Box::new(self.slots[0].clone()), Box::new(self.slots[1].clone())),
                        wavetables::tree::Tree::Mul(Box::new(self.slots[0].clone()), Box::new(self.slots[1].clone())),
                        wavetables::tree::Tree::Mirror(Box::new(self.slots[0].clone())),
                    ] {
                        row = row.push(make_canvas(f.build()));
                    }
                    row
                });
            },
        }
        Scrollable::new(column)
            // .horizontal_scroll(Default::default())
            .into()
    }
}

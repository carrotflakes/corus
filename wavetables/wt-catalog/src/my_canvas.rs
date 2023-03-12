use iced::{
    widget::canvas::{self, Frame, Path, Program, Stroke},
    Color, Point, Theme,
};

use crate::Message;

pub struct MyCanvas {
    cache: canvas::Cache,
    f: Box<dyn Fn(f64) -> f64>,
}

impl MyCanvas {
    pub fn new(f: Box<dyn Fn(f64) -> f64>) -> Self {
        Self {
            cache: canvas::Cache::new(),
            f,
        }
    }
}

impl Program<Message> for MyCanvas {
    type State = canvas::Cache;

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: iced::widget::canvas::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let size = bounds.size();
        let content = self.cache.draw(size, |frame: &mut Frame| {
            frame.fill_rectangle(Point::default(), size, Color::from_rgb(0.9, 0.9, 0.9));

            let path = Path::new(|b| {
                b.move_to(Point::new(
                    0.0,
                    (-(self.f)(0.0) / 2.0 + 0.5) as f32 * size.height,
                ));
                for i in 1..64 {
                    let x = i as f32 / 64.0 * size.width;
                    let y = (-(self.f)(i as f64 / 64.0) / 2.0 + 0.5) as f32 * size.height;
                    b.line_to(Point::new(x, y));
                }
            });
            frame.stroke(&path, Stroke::default().with_width(2.0));
        });
        vec![content]
    }
}

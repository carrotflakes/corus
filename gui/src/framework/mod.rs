mod keycode_map;

use std::path::Path;

use crate::interface;

pub struct Root;

impl interface::Root for Root {
    type Ui = Ui;

    type VideoSubsystem = <Self::Ui as interface::Ui>::VideoSubsystem;

    type AudioSubsystem = <Self::Ui as interface::Ui>::AudioSubsystem;

    type Window = Window;

    type Canvas = Canvas;

    type Font = <Self::Canvas as interface::Canvas>::Font;
}

pub struct Ui {
    sdl: sdl2::Sdl,
    event_pump: sdl2::EventPump,
    ttf_context: sdl2::ttf::Sdl2TtfContext,
}

impl interface::Ui for Ui {
    type VideoSubsystem = VideoSubsystem;

    type AudioSubsystem = AudioSubsystem;

    fn init() -> Result<Self, String> {
        sdl2::init().map(|x| Ui {
            event_pump: x.event_pump().unwrap(),
            sdl: x,
            ttf_context: sdl2::ttf::init().unwrap(),
        })
    }

    fn video(&mut self) -> Result<Self::VideoSubsystem, String> {
        self.sdl.video().map(|x| VideoSubsystem(x))
    }

    fn audio(&mut self) -> Result<Self::AudioSubsystem, String> {
        self.sdl.audio().map(|x| AudioSubsystem(x))
    }

    fn load_font(&mut self, path: impl AsRef<Path>, point_size: u16) -> Result<<<<Self::VideoSubsystem as interface::VideoSubsystem>::Window as interface::Window>::Canvas as interface::Canvas>::Font, String> {
        self.ttf_context
            .load_font(path, point_size)
            .map(|x| Font::new(unsafe { std::mem::transmute::<_, sdl2::ttf::Font<'static, 'static>>(x)}))
        //.map_err(|e| e.to_string())
    }

    fn events(&mut self) -> Vec<interface::Event> {
        use sdl2::event;
        self.event_pump.poll_iter().map(|e| match e {
            event::Event::Quit { timestamp } => {
                interface::Event::Quit {timestamp}
            }
            event::Event::KeyDown { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                interface::Event::KeyDown { timestamp, window_id, keycode: keycode.map(keycode_map::keycode_map), repeat}
            }
            event::Event::KeyUp { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                interface::Event::KeyUp { timestamp, window_id, keycode: keycode.map(keycode_map::keycode_map), repeat}
            }
            event::Event::TextEditing { timestamp, window_id, text, start, length } => {
                interface::Event::TextEditing {
                    timestamp,
                    window_id,
                    text,
                    start,
                    length,
                }
            }
            event::Event::TextInput { timestamp, window_id, text } => {
                interface::Event::TextInput {
                    timestamp,
                    window_id,
                    text,
                }
            }
            event::Event::MouseMotion { timestamp, window_id, which, mousestate, x, y, xrel, yrel } => {
                interface::Event::MouseMotion {
                    timestamp,
                    window_id,
                    which,
                    x,
                    y,
                    xrel,
                    yrel,

                }
            }
            event::Event::MouseButtonDown { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                interface::Event::MouseButtonDown {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn: mouse_btn_map(mouse_btn),
                    clicks,
                    x,
                    y,

                }
            }
            event::Event::MouseButtonUp { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                interface::Event::MouseButtonUp {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn: mouse_btn_map(mouse_btn),
                    clicks,
                    x,
                    y,

                }
            }
            event::Event::MouseWheel { timestamp, window_id, which, x, y, direction } => {
                interface::Event::MouseWheel {
                    timestamp,
                    window_id,
                    which,
                    x,
                    y,

                }
            }
            event::Event::FingerDown { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => {
                interface::Event::FingerDown {
                    timestamp,
                    touch_id,
                    finger_id,
                    x,
                    y,
                    dx,
                    dy,
                    pressure,

                }
            }
            event::Event::FingerUp { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => {
                interface::Event::FingerUp {
                    timestamp,
                    touch_id,
                    finger_id,
                    x,
                    y,
                    dx,
                    dy,
                    pressure,

                }
            }
            event::Event::FingerMotion { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => {
                interface::Event::FingerMotion {
                    timestamp,
                    touch_id,
                    finger_id,
                    x,
                    y,
                    dx,
                    dy,
                    pressure,

                }
            }
            _ => {
                interface::Event::Unknown
            }
        }).collect()
    }
}

fn mouse_btn_map(mouse_btn: sdl2::mouse::MouseButton) -> interface::MouseButton {
    match mouse_btn {
        sdl2::mouse::MouseButton::Unknown => interface::MouseButton::Unknown,
        sdl2::mouse::MouseButton::Left => interface::MouseButton::Left,
        sdl2::mouse::MouseButton::Middle => interface::MouseButton::Middle,
        sdl2::mouse::MouseButton::Right => interface::MouseButton::Right,
        sdl2::mouse::MouseButton::X1 => interface::MouseButton::X1,
        sdl2::mouse::MouseButton::X2 => interface::MouseButton::X2,
    }
}

pub struct VideoSubsystem(sdl2::VideoSubsystem);

impl interface::VideoSubsystem for VideoSubsystem {
    type Window = Window;

    fn new_window(&mut self, title: &str, width: u32, height: u32) -> Result<Self::Window, String> {
        self.0
            .window(title, width, height)
            .position_centered()
            .resizable()
            .build()
            .map(|x| Window { window: x })
            .map_err(|e| e.to_string())
    }

    fn text_input_start(&mut self, rect: interface::Rect) {
        let ti = self.0.text_input();
        ti.set_rect(sdl2::rect::Rect::new(rect.0, rect.1, rect.2, rect.3));
        ti.start();
    }
}

pub struct AudioSubsystem(sdl2::AudioSubsystem);

impl interface::AudioSubsystem for AudioSubsystem {
    fn open_playback<A: interface::AudioCallback>(
        &mut self,
        get_callback: &mut dyn FnMut(i32) -> A,
    ) -> Result<Box<dyn interface::AudioDevice<A>>, String> {
        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None, // default sample size
        };

        self.0
            .open_playback(
                None,
                &desired_spec,
                &mut move |spec: sdl2::audio::AudioSpec| AudioCallback(get_callback(spec.freq)),
            )
            .map(|mut x| {
                x.resume();
                Box::new(AudioDevice { device: x }) as Box<dyn interface::AudioDevice<A> + 'static>
            })
            .map_err(|e| e.to_string())
    }
}

pub struct AudioDevice<A: interface::AudioCallback> {
    device: sdl2::audio::AudioDevice<AudioCallback<A>>,
}

impl<A: interface::AudioCallback> interface::AudioDevice<A> for AudioDevice<A> {
}

pub struct AudioCallback<A: interface::AudioCallback>(A);

impl<A: interface::AudioCallback> sdl2::audio::AudioCallback for AudioCallback<A> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        self.0.callback(out);
    }
}

pub struct Window {
    window: sdl2::video::Window,
}

impl interface::Window for Window {
    type Canvas = Canvas;

    fn into_canvas(self) -> Result<Self::Canvas, String> {
        self.window
            .into_canvas()
            .build()
            .map(|x| Canvas { canvas: x })
            .map_err(|e| e.to_string())
    }
}

pub struct Canvas {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl interface::Canvas for Canvas {
    type Font = Font;

    fn draw_line(&mut self, start: interface::Point, end: interface::Point) -> Result<(), String> {
        self.canvas.draw_line((start.0, start.1), (end.0, end.1))
    }

    fn set_draw_color(&mut self, rgb: interface::RGB) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(rgb.0, rgb.1, rgb.2));
    }

    fn draw_rect(&mut self, rect: interface::Rect) {
        self.canvas
            .draw_rect(sdl2::rect::Rect::new(rect.0, rect.1, rect.2, rect.3))
            .unwrap();
    }

    fn draw_text(&mut self, font: &mut Font, text: &str, x: i32, y: i32) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
        }
        let texture_creator = self.canvas.texture_creator();
        font.0.set_style(sdl2::ttf::FontStyle::NORMAL);
        let surface = font
            .0
            .render(text)
            .blended(sdl2::pixels::Color::BLACK)
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let texture_size = surface.size();

        self.canvas
            .copy(
                &texture,
                None,
                Some(sdl2::rect::Rect::new(x, y, texture_size.0, texture_size.1)),
            )
            .unwrap();
        texture_size
    }

    fn clear(&mut self) {
        self.canvas.clear();
    }

    fn present(&mut self) {
        self.canvas.present();
    }
}

pub struct Font(sdl2::ttf::Font<'static, 'static>);

impl Font {
    fn new(font: sdl2::ttf::Font<'static, 'static>) -> Self {
        Font(font)
    }
}

impl interface::Font for Font {}

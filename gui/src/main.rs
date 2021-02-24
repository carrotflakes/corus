mod audio;

use std::{thread, time::Duration};

use corus::{
    node::{
        add::Add, amp::Amp, constant::Constant, controllable::Controllable, param::Param,
        proc_once_share::ProcOnceShare, sine::Sine,
    },
    notenum_to_frequency,
};
use sdl2::{
    audio::AudioSpecDesired, event::Event, keyboard::Keycode, pixels::Color, rect::Rect,
    render::Canvas, video::Window,
};

fn main() {
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsys = sdl_ctx.video().unwrap();
    let audio_subsys = sdl_ctx.audio().unwrap();
    let window = video_subsys
        .window("SDL2", 640, 480)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let osc_freq = Controllable::new(Param::new());
    let mod_freq_rate = Controllable::new(Param::new());
    let mod_gain = Controllable::new(Param::new());
    let mut osc_freq_ctrl = osc_freq.controller();
    let mut mod_freq_rate_ctrl = mod_freq_rate.controller();
    let mut mod_gain_ctrl = mod_gain.controller();
    osc_freq_ctrl.lock().set_value_at_time(0.0, 440.0);

    let mut device = audio_subsys
        .open_playback(None, &desired_spec, move |spec| {
            let osc_freq = ProcOnceShare::new(osc_freq);
            audio::Audio::new(
                spec.freq as u64,
                Box::new(Amp::new(
                    Sine::new(Add::new(
                        osc_freq.clone(),
                        Amp::new(
                            Sine::new(Amp::new(osc_freq.clone(), mod_freq_rate)),
                            mod_gain,
                        ),
                    )),
                    Constant::from(0.2),
                )),
            )
        })
        .unwrap();
    device.resume();

    let texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut font = ttf_context.load_font("./clacon.ttf", 20).unwrap();
    let mut draw_text = |canvas: &mut Canvas<Window>, text: &str, x: i32, y: i32| {
        font.set_style(sdl2::ttf::FontStyle::NORMAL);
        let surface = font
            .render(text)
            .blended(Color::RGB(0, 0, 0))
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let texture_size = surface.size();

        canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(x, y, texture_size.0, texture_size.1)),
            )
            .unwrap();
    };

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    'running: loop {
        let audio_time = device.lock().ctx.time;
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        // canvas.set_draw_color(Color::RGB(0, 255, 0));
        // canvas.fill_rect(Rect::new(rect_x, 0, 10, 10)).unwrap();

        draw_text(
            &mut canvas,
            format!("osc gain: {:>7.3}", 1.0).as_str(),
            10,
            10,
        );
        draw_text(
            &mut canvas,
            format!(
                "osc freq: {:>7.3} Hz",
                osc_freq_ctrl.lock().compute_value(audio_time)
            )
            .as_str(),
            10,
            30,
        );
        draw_text(
            &mut canvas,
            format!(
                "mod gain: {:>7.3}",
                mod_gain_ctrl.lock().compute_value(audio_time)
            )
            .as_str(),
            10,
            50,
        );
        draw_text(
            &mut canvas,
            format!(
                "mod freq rate: {:>7.3}",
                mod_freq_rate_ctrl.lock().compute_value(audio_time)
            )
            .as_str(),
            10,
            70,
        );
        canvas.present();

        for ev in event_pump.poll_iter() {
            match ev {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    let mut set = |nn: u32| {
                        osc_freq_ctrl
                            .lock()
                            .set_value_at_time(audio_time, notenum_to_frequency(nn));
                    };
                    match keycode {
                        Keycode::Z => set(64),
                        Keycode::S => set(65),
                        Keycode::X => set(66),
                        Keycode::D => set(67),
                        Keycode::C => set(68),
                        Keycode::V => set(69),
                        Keycode::G => set(70),
                        Keycode::B => set(71),
                        Keycode::H => set(72),
                        Keycode::N => set(73),
                        Keycode::J => set(74),
                        Keycode::M => set(75),
                        Keycode::Comma => set(76),
                        _ => (),
                    }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {}
                Event::MouseMotion { x, y, .. } => {
                    mod_freq_rate_ctrl
                        .lock()
                        .set_value_at_time(audio_time, y as f64 * 0.01);
                    mod_gain_ctrl
                        .lock()
                        .set_value_at_time(audio_time, x as f64 * 4.0);
                }
                _ => {}
            }
        }
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

use std::{thread, time::Duration};

use corus::{
    contrib::{
        controllable_param,
        generic_poly_synth::{NoteOff, NoteOn, PolySynth, Voice},
        rand_fm_synth::rand_fm_synth,
    },
    core::{
        amp::Amp, constant::Constant, controllable::Controllable, placeholder::Placeholder, Node,
    },
    notenum_to_frequency,
};
use sdl2::{
    audio::AudioSpecDesired, event::Event, keyboard::Keycode, pixels::Color, rect::Rect,
    render::Canvas, video::Window,
};

pub fn rand_fm() {
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

    let node = Controllable::new(Placeholder::new(Some(
        Box::new(Constant::new(0.0)) as Box<dyn Node<f64>>
    )));
    let mut node_ctrl = node.controller();

    let mut device = audio_subsys
        .open_playback(None, &desired_spec, move |spec| {
            crate::audio::Audio::new(spec.freq as u64, Box::new(node))
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
    let mut seed = 0;

    let synth = Controllable::new(create_fm_synth(seed));
    let mut synth_ctrl = synth.controller();
    node_ctrl.lock().set(Box::new(synth));
    let mut notenum_ons = vec![false; 128];

    'running: loop {
        let audio_time = device.lock().ctx.current_time;
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        draw_text(&mut canvas, &format!("seed: {:?}", seed), 5, 5);
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
                    let mut set = |nn: u8| {
                        if !notenum_ons[nn as usize] {
                            synth_ctrl
                                .lock()
                                .note_on(audio_time, Some(nn as u8), (nn as u8, 1.0));
                            notenum_ons[nn as usize] = true;
                        }
                    };
                    match keycode {
                        Keycode::W => {
                            seed += 1;
                            let synth = Controllable::new(create_fm_synth(seed));
                            synth_ctrl = synth.controller();
                            node_ctrl.lock().set(Box::new(synth) as Box<dyn Node<f64>>);
                        }
                        Keycode::Q => {
                            seed -= 1;
                            let synth = Controllable::new(create_fm_synth(seed));
                            synth_ctrl = synth.controller();
                            node_ctrl.lock().set(Box::new(synth) as Box<dyn Node<f64>>);
                        }
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
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    let mut set = |nn: u8| {
                        if notenum_ons[nn as usize] {
                            synth_ctrl.lock().note_off(audio_time, Some(nn as u8), ());
                            notenum_ons[nn as usize] = false;
                        }
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
                Event::MouseMotion { x, y, .. } => {}
                _ => {}
            }
        }
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

type MyVoice = Voice<Box<dyn Node<f64> + Send + Sync>, (u8, f64), ()>;

fn create_fm_synth(seed: u32) -> PolySynth<(u8, f64), (), MyVoice, Option<u8>> {
    PolySynth::new(
        &mut || {
            let (gain, mut gain_ctrl) = controllable_param(1.0);
            let synth = Controllable::new(rand_fm_synth(seed));
            let mut ctrl1 = synth.controller();
            let mut ctrl2 = synth.controller();
            let node = Amp::new(synth, gain);
            Voice(
                Box::new(node) as Box<dyn Node<f64> + Send + Sync>,
                Box::new(move |time, NoteOn((notenum, velocity))| {
                    gain_ctrl.lock().set_value_at_time(time, velocity);
                    ctrl1
                        .lock()
                        .note_on(time, notenum_to_frequency(notenum as u32));
                }),
                Box::new(move |time, NoteOff(())| {
                    ctrl2.lock().note_off(time);
                }),
            )
        },
        16,
    )
}

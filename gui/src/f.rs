use std::{
    any::Any,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use corus::{
    core::{var::Var, controllable::Controllable, mul::Mul, share::Share, sine::Sine},
    notenum_to_frequency,
    signal::Mono,
    time::Sample,
    Node, ProcContext,
};

use crate::{
    component::{
        text_box::TextBox, text_box_container::TextBoxContainer, text_input_state::TextInputState,
        Component,
    },
    context::Context,
    interface::{VideoSubsystem, *},
};

pub fn f<U: Ui>() {
    let mut ui = U::init().unwrap();
    let mut video_subsys = ui.video().unwrap();
    let mut audio_subsys = ui.audio().unwrap();
    let window = video_subsys.new_window("SDL2", 640, 480).unwrap();
    let canvas = window.into_canvas().unwrap();

    let audio_ctx = Arc::new(Mutex::new(ProcContext::new(44100 as u64)));
    let controllable = Controllable::new(Sine::new(Var::from(440.0)));
    let mut controller = controllable.controller();
    let controllable = Share::new(controllable);

    let mut device = audio_subsys
        .open_playback(&mut |sample_rate| {
            *audio_ctx.lock().unwrap() = ProcContext::new(sample_rate as u64);
            Audio::new(
                audio_ctx.clone(),
                Box::new(Mul::new(controllable.clone(), Var::from(0.05))),
            )
        })
        .unwrap();

    let font = ui.load_font("./clacon.ttf", 20).unwrap();

    let text_inputing_state = TextInputState::new(false, "".to_string());
    let mut ctx = Context::<U>::new(canvas, font, text_inputing_state, video_subsys);

    let mut my_component = TextBoxContainer::new(vec![
        TextBox::new(0, "hello".to_string(), Rect::new(100, 100, 100, 20)),
        TextBox::new(1, "world".to_string(), Rect::new(100, 130, 100, 20)),
        TextBox::new(2, "!!!!".to_string(), Rect::new(100, 160, 100, 20)),
    ]);

    'running: loop {
        let audio_time = audio_ctx.lock().unwrap().current_time;
        ctx.canvas.set_draw_color(RGB(255, 255, 255));
        ctx.canvas.clear();

        my_component.draw(&mut ctx);
        ctx.text_inputing_state
            .clone()
            .lock()
            .unwrap()
            .draw(&mut ctx);
        ctx.canvas.present();

        'event_receive: for ev in ui.events() {
            {
                if ctx
                    .text_inputing_state
                    .clone()
                    .lock()
                    .unwrap()
                    .event_receive(&mut ctx, &ev)
                {
                    continue 'event_receive;
                }
            }
            match &ev {
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
                        // osc_freq_ctrl
                        //     .lock()
                        //     .set_value_at_time(audio_time, notenum_to_frequency(nn));
                        *controller.lock() = Sine::new(Var::from(notenum_to_frequency(nn)));
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
                        Keycode::Return => {}
                        _ => (),
                    }
                }
                Event::MouseButtonDown { .. } => {}
                Event::MouseMotion { x, y, .. } => {
                    // mod_freq_rate_ctrl
                    //     .lock()
                    //     .set_value_at_time(audio_time, *y as f64 * 0.01);
                    // mod_gain_ctrl
                    //     .lock()
                    //     .set_value_at_time(audio_time, *x as f64 * 4.0);
                }
                _ => {}
            }
            let message = Box::new(ev.clone()) as Box<dyn Any>;
            if my_component.message_receive(&mut ctx, &message) {
                continue 'event_receive;
            }
        }
        let mut messages = Vec::new();
        std::mem::swap(&mut messages, &mut ctx.messages);
        for message in messages {
            my_component.message_receive(&mut ctx, &message);
        }
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

pub struct Audio {
    node: Box<dyn Node<Output = f64> + Send + Sync>,
    pub ctx: Arc<Mutex<ProcContext>>,
}

impl Audio {
    pub fn new(ctx: Arc<Mutex<ProcContext>>, node: Box<dyn Node<Output = f64> + Send + Sync>) -> Self {
        Self { node, ctx }
    }
}

impl AudioCallback for Audio {
    fn callback(&mut self, out: &mut [f32]) {
        let mut ctx = self.ctx.lock().unwrap();
        let mut s = ctx.lock(&mut self.node, Sample(out.len() as u64));
        for x in out.iter_mut() {
            *x = s.next().unwrap().get_m() as f32;
        }
    }
}

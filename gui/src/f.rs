use std::{
    any::Any,
    borrow::BorrowMut,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use corus::{
    core::{
        add::Add, amp::Amp, constant::Constant, controllable::Controllable, param::Param,
        share::Share, sine::Sine,
    },
    notenum_to_frequency,
    signal::Mono,
    time::Sample,
    Node, ProcContext,
};

use crate::{component::{Component, text_input_state::TextInputState, text_box::TextBox, component_container::ComponentContainer}, context::Context, interface::{VideoSubsystem, *}};

pub fn f<U: Ui>() {
    let mut ui = U::init().unwrap();
    let mut video_subsys = ui.video().unwrap();
    let mut audio_subsys = ui.audio().unwrap();
    let mut window = video_subsys.new_window("SDL2", 640, 480).unwrap();
    let canvas = window.into_canvas().unwrap();

    // let desired_spec = AudioSpecDesired {
    //     freq: Some(44100),
    //     channels: Some(1), // mono
    //     samples: None,     // default sample size
    // };

    // let osc_freq = Controllable::new(Param::new());
    // let mod_freq_rate = Controllable::new(Param::new());
    // let mod_gain = Controllable::new(Param::new());
    // let mut osc_freq_ctrl = osc_freq.controller();
    // let mut mod_freq_rate_ctrl = mod_freq_rate.controller();
    // let mut mod_gain_ctrl = mod_gain.controller();
    // osc_freq_ctrl.lock().set_value_at_time(0.0, 440.0);

    let mut audio_ctx = Arc::new(Mutex::new(ProcContext::new(44100 as u64)));

    let mut device = audio_subsys
        .open_playback(&mut move |sample_rate| {
            // let osc_freq = Share::new(osc_freq);
            *audio_ctx.lock().unwrap() = ProcContext::new(sample_rate as u64);
            Audio::new(
                audio_ctx.clone(),
                Box::new(Amp::new(
                    Sine::new(
                        // osc_freq.clone(),
                        Constant::from(440.0),
                    ),
                    Constant::from(0.05),
                )),
            )
        })
        .unwrap();

    let font = ui.load_font("./clacon.ttf", 20).unwrap();

    let text_inputing_state = TextInputState::new(false, "".to_string());
    let mut ctx = Context::<U>::new(
        canvas,
        font,
        text_inputing_state,
        video_subsys,
    );

    // let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut my_component = ComponentContainer::new(vec![
        Box::new(TextBox::new(
            "hello".to_string(),
            Rect::new(100, 100, 100, 20),
        )),
        Box::new(TextBox::new(
            "world".to_string(),
            Rect::new(100, 130, 100, 20),
        )),
        Box::new(TextBox::new(
            "!!!!".to_string(),
            Rect::new(100, 160, 100, 20),
        )),
    ]);

    'running: loop {
        let audio_time = 0.0;// device.lock().ctx.current_time;
        ctx.canvas.set_draw_color(RGB(255, 255, 255));
        ctx.canvas.clear();

        // canvas.set_draw_color(Color::RGB(0, 255, 0));
        // canvas.fill_rect(Rect::new(rect_x, 0, 10, 10)).unwrap();

        // ctx.draw_text(format!("osc gain: {:>7.3}", 1.0).as_str(), 10, 10);
        // ctx.draw_text(
        //     format!(
        //         "osc freq: {:>7.3} Hz",
        //         osc_freq_ctrl.lock().compute_value(audio_time)
        //     )
        //     .as_str(),
        //     10,
        //     30,
        // );
        // ctx.draw_text(
        //     format!(
        //         "mod gain: {:>7.3}",
        //         mod_gain_ctrl.lock().compute_value(audio_time)
        //     )
        //     .as_str(),
        //     10,
        //     50,
        // );
        // ctx.draw_text(
        //     format!(
        //         "mod freq rate: {:>7.3}",
        //         mod_freq_rate_ctrl.lock().compute_value(audio_time)
        //     )
        //     .as_str(),
        //     10,
        //     70,
        // );
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
                    // let mut set = |nn: u32| {
                    //     osc_freq_ctrl
                    //         .lock()
                    //         .set_value_at_time(audio_time, notenum_to_frequency(nn));
                    // };
                    // match keycode {
                    //     Keycode::Z => set(64),
                    //     Keycode::S => set(65),
                    //     Keycode::X => set(66),
                    //     Keycode::D => set(67),
                    //     Keycode::C => set(68),
                    //     Keycode::V => set(69),
                    //     Keycode::G => set(70),
                    //     Keycode::B => set(71),
                    //     Keycode::H => set(72),
                    //     Keycode::N => set(73),
                    //     Keycode::J => set(74),
                    //     Keycode::M => set(75),
                    //     Keycode::Comma => set(76),
                    //     Keycode::Return => {}
                    //     _ => (),
                    // }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {}
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
    node: Box<dyn Node<f64> + Send + Sync>,
    pub ctx: Arc<Mutex<ProcContext>>,
}

impl Audio {
    pub fn new(ctx: Arc<Mutex<ProcContext>>, node: Box<dyn Node<f64> + Send + Sync>) -> Self {
        Self {
            node,
            ctx,
        }
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

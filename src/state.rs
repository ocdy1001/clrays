use crate::scene::Scene;
use crate::vec3::Vec3;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::f32::consts::{ PI, FRAC_PI_2 };

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoopRequest{
    Continue,
    Stop,
}

const KEYS_AMOUNT: usize = 11;
pub type Keymap = [Keycode; KEYS_AMOUNT];

#[macro_export]
macro_rules! build_keymap{
    ($mfo:ident,$mba:ident,$mle:ident,$mri:ident,$mup:ident,$mdo:ident,
     $lup:ident,$ldo:ident,$lle:ident,$lri:ident,$foc:ident) => {
        [Keycode::$mfo, Keycode::$mba, Keycode::$mle, Keycode::$mri, Keycode::$mup, Keycode::$mdo,
         Keycode::$lup, Keycode::$ldo, Keycode::$lle, Keycode::$lri, Keycode::$foc]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderMode{
    Full,
    Reduced,
    None,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings{
    pub aa_samples: usize,
    pub max_reduced_ms: f32,
    pub start_in_focus_mode: bool,
}

impl Default for Settings{
    fn default() -> Self{
        Self{
            aa_samples: 4,
            max_reduced_ms: 50.0,
            start_in_focus_mode: false,
        }
    }
}

impl Settings{
    pub fn start_aa(&self) -> usize{
        if self.start_in_focus_mode{
            self.aa_samples
        } else {
            1
        }
    }
}

#[derive(Clone, Debug)]
pub struct State{
    pub key_map: Keymap,
    keys: [bool; KEYS_AMOUNT],
    pub render_mode: RenderMode,
    pub last_frame: RenderMode,
    pub reduced_rate: usize,
    pub aa: usize,
    pub aa_count: usize,
    pub settings: Settings,
}

impl State{
    pub fn new(key_map: Keymap, settings: Settings) -> Self{
        Self{
            key_map,
            keys: [false; KEYS_AMOUNT],
            render_mode: RenderMode::Reduced,
            last_frame: RenderMode::None,
            reduced_rate: 2,
            aa: settings.start_aa(),
            aa_count: 0,
            settings,
        }
    }

    pub fn toggle_focus_mode(&mut self){
        self.aa_count = 0;
        self.render_mode = RenderMode::Reduced;
        self.aa = if self.aa == 1 {
            self.settings.aa_samples
        } else {
            1
        }
    }
}

pub type InputFn = fn (_events: &[Event], _scene: &mut Scene, _state: &mut State) -> LoopRequest;
pub type UpdateFn = fn (_dt: f32, _state: &mut State) -> LoopRequest;

pub fn std_update_fn(_: f32, _state: &mut State) -> LoopRequest { LoopRequest::Continue }

pub fn log_update_fn(dt: f32, state: &mut State) -> LoopRequest {
    if state.last_frame != RenderMode::None{
        if state.last_frame == RenderMode::Reduced{
            println!("{:?}({}): {} ms, ", state.last_frame, state.reduced_rate, dt);
        }
        if state.last_frame == RenderMode::Reduced && dt > state.settings.max_reduced_ms{
            state.reduced_rate += 1;
        }
    }
    LoopRequest::Continue
}

pub fn std_input_fn(events: &[Event], _: &mut Scene, _: &mut State) -> LoopRequest{
    for event in events.iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return LoopRequest::Stop;
            },
            _ => {}
        }
    }
    LoopRequest::Continue
}

pub fn fps_input_fn(events: &[Event], scene: &mut Scene, state: &mut State) -> LoopRequest{
    fn yaw_roll(yaw: f32, roll: f32) -> Vec3 {
        let a = roll;  // Up/Down
        let b = yaw;   // Left/Right
        Vec3 { x: a.cos() * b.sin(), y: a.sin(), z: -a.cos() * b.cos() }
    }

    let cam = &mut scene.cam;
    let old_pos = cam.pos;
    let old_dir = cam.dir;

    for event in events.iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return LoopRequest::Stop;
            },
            Event::KeyDown { keycode: Some(x), .. } if *x == state.key_map[10] => {
                state.toggle_focus_mode();
            },
            Event::KeyDown { keycode: Some(x), repeat: false, .. } => {
                for (i, binding) in state.key_map.iter().enumerate(){
                    if x == binding{
                        state.keys[i] = true;
                    }
                }
            },
            Event::KeyUp { keycode: Some(x), repeat: false, .. } => {
                for (i, binding) in state.key_map.iter().enumerate(){
                    if x == binding{
                        state.keys[i] = false;
                    }
                }
            },
            _ => {},
        }
    }
    let ms = cam.move_sensitivity;
    let ls = cam.look_sensitivity;
    for (i, active) in state.keys.iter().enumerate(){
        if !active { continue; }
        match i {
            0 => { // Move Forward; Move into camera direction
                cam.pos.add(cam.dir.scaled(ms));
            },
            1 => { // Move Backward; Move opposite camera direction
                cam.pos.add(cam.dir.neged().scaled(ms));
            },
            2 => { // Move Left; Move camera direction crossed z-axis, negated
                cam.pos.add(cam.dir.crossed(Vec3::UP).neged().scaled(ms));
            },
            3 => { // Move Right; Move camera direction crossed z-axis
                cam.pos.add(cam.dir.crossed(Vec3::UP).scaled(ms));
            },
            4 => { // Move Up; Move camera direction crossed x-axis
                cam.pos.add(cam.dir.crossed(Vec3::RIGHT).scaled(ms));
            },
            5 => { // Move Down; Move camera direction crossed x-axis
                cam.pos.add(cam.dir.crossed(Vec3::RIGHT).neged().scaled(ms));
            },
            6 => { // Look Up;
                cam.ori[1] = (cam.ori[1] + ls).min(FRAC_PI_2).max(-FRAC_PI_2);
                let yaw = cam.ori[0]; // Up/Down
                let roll = cam.ori[1]; // Left/Right
                cam.dir = yaw_roll(yaw, roll);
            },
            7 => { // Look Down;
                cam.ori[1] = (cam.ori[1] - ls).min(FRAC_PI_2).max(-FRAC_PI_2);
                let yaw = cam.ori[0]; // Up/Down
                let roll = cam.ori[1]; // Left/Right
                cam.dir = yaw_roll(yaw, roll);
            },
            8 => { // Look Left;
                cam.ori[0] -= ls;
                if cam.ori[0] < -PI {
                    cam.ori[0] += 2.0 * PI;
                }
                let yaw = cam.ori[0]; // Up/Down
                let roll = cam.ori[1]; // Left/Right
                cam.dir = yaw_roll(yaw, roll);
            },
            9 => { // Look Right;
                cam.ori[0] += ls;
                if cam.ori[0] > PI {
                    cam.ori[0] -= 2.0 * PI;
                }
                let yaw = cam.ori[0]; // Up/Down
                let roll = cam.ori[1]; // Left/Right
                cam.dir = yaw_roll(yaw, roll);
            },
            _ => {},
        }
    }
    let moved = old_pos != cam.pos || old_dir != cam.dir;
    state.render_mode = match (moved, state.render_mode){
        (true, _) => RenderMode::Reduced,
        (false, RenderMode::Reduced) => RenderMode::Full,
        _ => RenderMode::None,
    };
    LoopRequest::Continue
}

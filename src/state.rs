use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub trait State{
    fn new() -> Self;
    fn should_close(&self) -> bool;
    fn handle_input(&mut self, event_pump: &mut sdl2::EventPump);
    fn update(&mut self, dt: f64);
}

#[derive(Clone,Default)]
pub struct StdState{
    should_close: bool,
}

impl State for StdState{
    fn new() -> Self{
        Self::default()
    }

    fn should_close(&self) -> bool{
        self.should_close
    }

    fn handle_input(&mut self, event_pump: &mut sdl2::EventPump){
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    self.should_close = true;
                    break;
                },
                _ => {}
            }
        }
    }

    fn update(&mut self, _dt: f64){ }
}

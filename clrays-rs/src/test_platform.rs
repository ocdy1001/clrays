pub enum PlatformTest{
    SdlWindow,
    SdlAudio,
    OpenCl,
}

pub fn run_platform_test(t: PlatformTest){
    match t{
        PlatformTest::SdlWindow => test_sdl_window(),
        PlatformTest::SdlAudio => test_sdl_audio(),
        PlatformTest::OpenCl => test_opencl(),
    }
}

pub fn test_sdl_window(){
    use sdl2::pixels::Color;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use std::time::Duration;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    let mut f = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        f += 1;
        if f > 1000 { break; }
    }
}

pub fn test_sdl_audio(){
    use sdl2::audio::{AudioCallback, AudioSpecDesired};
    use std::time::Duration;

    struct SquareWave {
        phase_inc: f32,
        phase: f32,
        volume: f32
    }
    
    impl AudioCallback for SquareWave {
        type Channel = f32;
    
        fn callback(&mut self, out: &mut [f32]) {
            // Generate a square wave
            for x in out.iter_mut() {
                *x = if self.phase <= 0.5 {
                    self.volume
                } else {
                    -self.volume
                };
                self.phase = (self.phase + self.phase_inc) % 1.0;
            }
        }
    }

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    }).unwrap();

    // Start playback
    device.resume();

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(10000));
}

pub fn test_opencl(){
    use ocl::ProQue;

    fn trivial() -> ocl::Result<()> {
        let src = r#"
            __kernel void add(__global float* buffer, float scalar) {
                buffer[get_global_id(0)] += scalar;
            }
        "#;

        let pro_que = ProQue::builder()
            .src(src)
            .dims(1 << 20)
            .build()?;

        let buffer = pro_que.create_buffer::<f32>()?;

        let kernel = pro_que.kernel_builder("add")
            .arg(&buffer)
            .arg(10.0f32)
            .build()?;

        unsafe { kernel.enq()?; }

        let mut vec = vec![0.0f32; buffer.len()];
        buffer.read(&mut vec).enq()?;

        println!("The value at index [{}] is now '{}'!", 200007, vec[200007]);
        Ok(())
    }

    let res = trivial();
    match res{
        Ok(_) => {},
        Err(e) => println!("{}", e),
    }
}

use crate::cl_helpers::{ create_five, ClBufferRW };

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{ AudioCallback, AudioSpecDesired };
use ocl::{ Buffer, flags, Kernel, ProQue };

use std::time::Duration;

pub enum PlatformTest{
    SdlWindow,
    SdlAudio,
    OpenCl0,
    OpenCl1,
    OpenCl2,
}

pub fn run_platform_test(t: PlatformTest){
    match t{
        PlatformTest::SdlWindow => test_sdl_window(),
        PlatformTest::SdlAudio => test_sdl_audio(),
        PlatformTest::OpenCl0 => test_opencl0(),
        PlatformTest::OpenCl1 => test_opencl1(),
        PlatformTest::OpenCl2 => test_opencl2(),
    }
}

pub fn test_sdl_window(){
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
        if f > 120 { break; }
    }
}

pub fn test_sdl_audio(){
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
            phase_inc: 110.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.1
        }
    }).unwrap();

    // Start playback
    device.resume();

    // Play for 2 seconds
    std::thread::sleep(Duration::from_millis(1000));
}

pub fn test_opencl0(){
    let src = r#"
        __kernel void add(__global float* buffer, float scalar) {
            buffer[get_global_id(0)] += scalar;
        }
    "#;

    let pro_que = ProQue::builder()
        .src(src)
        .dims(1 << 20)
        .build().unwrap();

    let buffer = pro_que.create_buffer::<f32>().unwrap();

    let kernel = pro_que.kernel_builder("add")
        .arg(&buffer)
        .arg(10.0f32)
        .build().unwrap();

    unsafe { kernel.enq().unwrap(); }

    let mut vec = vec![0.0f32; buffer.len()];
    buffer.read(&mut vec).enq().unwrap();

    println!("The value at index [{}] is now '{}'!", 200007, vec[200007]);
    println!("Test opencl0 went ok!");
}

pub fn test_opencl1(){
    fn run() -> ocl::Result<()>{
        let src = r#"
            __kernel void add(__global float* buffer, float scalar) {
                buffer[get_global_id(0)] += scalar;
            }
        "#;
        let (_,_,_,program,queue) = match create_five(src){
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        let dims = 1 << 20;
        let buffer = match Buffer::<f32>::builder()
        .queue(queue.clone())
        .flags(flags::MEM_READ_WRITE)
        .len(dims)
        .fill_val(0f32)
        .build(){
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        let kernel = match Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(dims)
        .arg(&buffer)
        .arg(&10f32)
        .build(){
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        unsafe {
            match kernel.cmd().queue(&queue).enq(){
                Ok(_) => {},
                Err(e) => return Err(e),
            }
        }
        let mut vec = vec![0f32; dims];
        match buffer.cmd()
        .queue(&queue)
        .read(&mut vec)
        .enq(){
            Ok(_) => {},
            Err(e) => return Err(e),
        }
        assert_eq!(vec, vec![10.0f32; dims]);
        println!("Test opencl1 went ok!");
        Ok(())
    }

    let res = run();
    match res{
        Ok(_) => {},
        Err(e) => println!("{}", e),
    }
}

pub fn test_opencl2(){
    let src = r#"
        __kernel void write(__global int* buffer) {
            buffer[get_global_id(0)] += get_global_id(0);
        }
    "#;
    let (_, _, _, program, queue) = create_five(src).unwrap();
    let dims = 1 << 12;
    let mut startingbuffer = vec![0i32; dims];
    for (i, buffer_elem) in startingbuffer.iter_mut().enumerate().take(dims){
        *buffer_elem = i as i32;
    }
    let mut clbuffer = ClBufferRW::from(&queue, startingbuffer).unwrap();
    let kernel = Kernel::builder()
        .program(&program)
        .name("write")
        .queue(queue.clone())
        .global_work_size(dims)
        .arg(clbuffer.get_ocl_buffer())
        .build().unwrap();

    unsafe {
        kernel.cmd().queue(&queue).enq().unwrap();
    }
    clbuffer.download(&queue).expect("expect: test_opencl2 clbuffer download");
    let mut testvec = vec![0i32; dims];
    for (i, buffer_elem) in testvec.iter_mut().enumerate().take(dims){
        *buffer_elem = (i * 2) as i32;
    }
    assert_eq!(clbuffer.get_slice(), testvec.as_slice());
    println!("Test opencl2 went ok!");
}

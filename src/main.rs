#![feature(duration_constants)]

mod chip8;
mod display;
use std::env;
use std::path::Path;
use tracing_log::LogTracer;

use pixels::{Error, Pixels, SurfaceTexture};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 320;

fn main() -> Result<(), Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    LogTracer::init().unwrap();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("elise is very cute!");

    let args: Vec<String> = env::args().collect();

    let mut c8 = chip8::Chip8::new(Path::new(&args[1]));

    //let mut canvas = Canvas::new(64, 32);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Chip8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut inputs = [false; 16];

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            let frame = pixels.get_frame();
            println!("uwu {:?}?", inputs);
            let c8frame = c8.run_frame(inputs);

            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let x = (i % WIDTH as usize) as i16;
                let y = (i / WIDTH as usize) as i16;
                let rgba = if c8frame[(y / 10) as usize][(x / 10) as usize] {
                    [0xff, 0xff, 0xff, 0xff]
                } else {
                    [0xff, 0x00, 0x00, 0xff]
                };

                pixel.copy_from_slice(&rgba);
            }

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Event::RedrawEventsCleared = event {
            window.request_redraw();
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            inputs = [
                VirtualKeyCode::Key1,
                VirtualKeyCode::Key2,
                VirtualKeyCode::Key3,
                VirtualKeyCode::Key4,
                VirtualKeyCode::Key5,
                VirtualKeyCode::Key6,
                VirtualKeyCode::Key7,
                VirtualKeyCode::Key8,
                VirtualKeyCode::Key9,
                VirtualKeyCode::Key0,
                VirtualKeyCode::Q,
                VirtualKeyCode::W,
                VirtualKeyCode::E,
                VirtualKeyCode::R,
                VirtualKeyCode::T,
                VirtualKeyCode::Y,
            ]
            .map(|k| input.key_held(k));

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
        }
    });

}

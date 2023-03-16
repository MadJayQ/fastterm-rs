use std::thread;

use byte_unit::n_kb_bytes;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};


use fastterm::{TerminalRenderer, Terminal};
#[macro_use] extern crate byte_unit;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();


    let mut terminal = Terminal::new(n_kb_bytes(16) as u64); //'a
    let mut terminal_renderer = pollster::block_on(TerminalRenderer::new(window, terminal)); //'b

    let terminal_thread = thread::Builder::new().name("TerminalThread".to_string()).spawn(move || {
        loop {
            
        }
    });

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == terminal_renderer.window().id() => {
                if !terminal_renderer.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            terminal_renderer.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            terminal_renderer.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == terminal_renderer.window().id() => {
                // terminal.update();
                match terminal_renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => terminal_renderer.resize(terminal_renderer.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                terminal_renderer.window().request_redraw();
            }
            _ => {}
        }
    });

    //'end b
    //'end a

}

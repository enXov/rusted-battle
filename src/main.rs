use anyhow::Result;
use log::info;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod core;
mod engine;
mod game;

use engine::renderer::Renderer;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting Rusted Battle...");

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rusted Battle")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .with_resizable(true)
            .build(&event_loop)?,
    );

    info!("Window created successfully");

    // Initialize renderer
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    info!("Renderer initialized successfully");

    // Main event loop
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("Close requested, shutting down...");
                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(physical_size),
                    ..
                } => {
                    info!("Window resized to {:?}", physical_size);
                    renderer.resize(physical_size);
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Render frame
                    if let Err(e) = renderer.render() {
                        log::error!("Render error: {:?}", e);
                        // For now, just log the error
                        // In a production app, we'd handle specific surface errors
                    }
                }
                Event::AboutToWait => {
                    // Request redraw on next frame
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;

    Ok(())
}

use winit::{
	event::Event,
	event_loop::EventLoop,
	window::{Window, WindowId},
};

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

mod components;
mod layout;
use layout::Layout;
use layout::WindowLifeStatus;

type InitialLayout = layout::DrawingWindow;

const FPS: i16 = 144;
const FRAMETIME: Duration = Duration::from_nanos(1_000_000_000 / (FPS as u64));

async fn run() {
	env_logger::init();

	let event_loop = EventLoop::new();
	let mut window_map = HashMap::<WindowId, Box<dyn Layout>>::new();
	let mut last_frame_time = HashMap::<WindowId, SystemTime>::new();

	// Start initial layout
	let ctx = InitialLayout::init();

	let window = Arc::new(Window::new(&event_loop).expect("Could not create window"));

	let mut initial_layout = InitialLayout::new(ctx, window).await;
	initial_layout.render();

	window_map.insert(initial_layout.window().id(), initial_layout);

	event_loop.run(move |event, event_loop, control_flow| {
		control_flow.set_wait();

		match event {
			Event::WindowEvent { window_id, event } => {
				match window_map.get_mut(&window_id) {
					None => {
						println!("Ignoring event to invalid window: {:?}", window_id);
						return;
					}
					Some(r) => r,
				}
				.event_handler(event);
			}
			Event::MainEventsCleared => {
				let mut should_remove: Vec<WindowId> = Vec::new();
				let mut should_add: Vec<Box<dyn Layout>> = Vec::new();
				window_map.values_mut().for_each(|layout| {
					let (window_state, child) = layout.update(event_loop);

					if let WindowLifeStatus::Dead = window_state {
						should_remove.push(layout.window().id());
					}

					if let Some(child_layout) = child {
						should_add.push(child_layout);
					}
				});

				for win_id in should_remove {
					window_map.remove(&win_id);
				}

				for child_layout in should_add {
					let child_window_id = child_layout.window().id();
					if window_map.contains_key(&child_window_id) {
						panic!("New window has the same Id as other alive window")
					}

					window_map.insert(child_window_id, child_layout);
				}

				if window_map.is_empty() {
					control_flow.set_exit_with_code(0);
				}
			}
			Event::RedrawRequested(window_id) => {
				if let Some(layout) = window_map.get_mut(&window_id) {
					if let Some(t) = last_frame_time.get(&window_id) {
						let elap = t.elapsed().unwrap();
						if elap < FRAMETIME {
							std::thread::sleep(FRAMETIME - elap);
						}
					}
					layout.render();
					last_frame_time.insert(window_id, SystemTime::now());
				}
			}
			_ => (),
		}
	})
}

fn main() {
	pollster::block_on(run());
}

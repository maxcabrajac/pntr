use winit::{
	event::Event,
	event_loop::EventLoop,
	window::{Window, WindowId},
};

use std::{collections::HashMap, sync::Arc};

mod components;
mod layout;
use layout::Layout;
use layout::WindowLifeStatus;

async fn run() {
	env_logger::init();

	let event_loop = EventLoop::new();
	let mut window_map = HashMap::<WindowId, Box<dyn Layout>>::new();

	// Start initial layout
	let window = Arc::new(Window::new(&event_loop).expect("Could not create window"));
	let initial_layout = layout::Triangle::new(window).await;

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
					layout.render();
				}
			}
			_ => (),
		}
	})
}

fn main() {
	pollster::block_on(run());
}

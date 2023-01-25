use crate::CustomEvents;
use winit::{
	window::WindowId,
	event_loop::EventLoop,
};
use std::{
	time::{SystemTime, Duration}, collections::HashMap,
};
use async_std::task;
use async_std::sync::{RwLock, Arc, Mutex};
use async_std::channel;
use crate::FRAMETIME;

pub struct FrameLimiter {
	sender: channel::Sender<WindowId>,
}

impl FrameLimiter {
	pub fn new(event_loop: &EventLoop<CustomEvents>) -> Self {

		let (sender, receiver) = channel::unbounded::<WindowId>();
		let proxy_mutex = Arc::new(Mutex::new(event_loop.create_proxy()));

		task::spawn(async move {
			let last_frame_map = Arc::new(RwLock::new(HashMap::<WindowId, (SystemTime, bool)>::new()));

			loop {
				match receiver.recv().await {
					Err(_) => {
						return;
					},
					Ok(wid) => {
						let task_last_frame_map = last_frame_map.clone();
						let task_proxy_mutex = proxy_mutex.clone();
						task::spawn(async move {
							let r = task_last_frame_map.read().await;
							let mut dur = Duration::ZERO;
							if let Some((t, b)) = r.get(&wid) {
								if *b { return; }
								if FRAMETIME > t.elapsed().unwrap() {
									dur = FRAMETIME - t.elapsed().unwrap();
								}
							};
							drop(r);

							if !dur.is_zero() {
								let mut w = task_last_frame_map.write().await;
								if let Some(mut val) = w.get_mut(&wid) {
									val.1 = true;
								} else {
									w.insert(wid.clone(), (SystemTime::now(), true));
								}
								drop(w);

								task::sleep(dur).await;
							}

							let mut w = task_last_frame_map.write().await;
							w.insert(wid, (SystemTime::now(), false));
							drop(w);

							let proxy = task_proxy_mutex.lock().await;
							if let Err(_) = proxy.send_event(CustomEvents::ShouldRedraw(wid)) {
								return;
							}
							drop(proxy);
						});
					}
				}
			}
		});

		FrameLimiter {
			sender,
		}
	}

	pub fn schedule_redraw(&self, wid: WindowId) {
		async_std::task::block_on(self.sender.send(wid)).unwrap();
	}
}

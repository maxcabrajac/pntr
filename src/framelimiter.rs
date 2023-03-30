use winit::{
	window::WindowId,
	event_loop::{EventLoop, EventLoopProxy},
};
use core::cmp::Reverse;
use std::{
	time::{SystemTime, Duration}, collections::{BinaryHeap, HashMap}, sync::mpsc, thread,
};

use crate::CustomEvents;
use crate::FRAMETIME;

pub struct FrameLimiter {
	sender: mpsc::Sender<WindowId>,
}

struct FrameSchedule {
	last_scheduled_frametime: HashMap<WindowId, SystemTime>,
	schedule_queue: BinaryHeap<Reverse<(SystemTime, WindowId)>>,
	event_proxy: EventLoopProxy<CustomEvents>,
}

#[inline(always)]
fn now() -> SystemTime {
	SystemTime::now()
}

impl FrameSchedule {
	pub fn new(event_proxy: EventLoopProxy<CustomEvents>) -> FrameSchedule {
		FrameSchedule {
			last_scheduled_frametime: HashMap::<WindowId, SystemTime>::new(),
			schedule_queue: BinaryHeap::<Reverse::<(SystemTime, WindowId)>>::new(),
			event_proxy
		}

	}

	pub fn time_to_next_frame(&self) -> Option<Duration> {
		match	self.schedule_queue.peek() {
			None => None,
			Some(Reverse((time_of_next_frame, _))) => {
				Some(time_of_next_frame.duration_since(now()).unwrap_or(Duration::ZERO))
			}
		}
	}

	pub fn insert(&mut self, wid: WindowId) {
		match self.last_scheduled_frametime.get(&wid) {
			Some(time) if *time > now() => {
				// next frame on this window is already scheduled, do nothing
			}

			Some(time) if *time + FRAMETIME > now()  => {
				// next frame on this window should be scheduled

				let next_frame_time = *time + FRAMETIME;
				self.last_scheduled_frametime.insert(wid, next_frame_time);
				self.schedule_queue.push(Reverse((next_frame_time, wid)));
			}

			_ => {
				// should draw imediately

				self.last_scheduled_frametime.insert(wid, now());
				self.send_redraw(&wid);
			}
		}
	}

	pub fn process_due_frames(&mut self) {
		while let Some(Reverse((time, wid))) = self.schedule_queue.peek() {
			if time > &now() {
				break;
			}

			self.send_redraw(wid);
			self.schedule_queue.pop();
		}
	}

	fn send_redraw(&self, wid: &WindowId) {
		self.event_proxy.send_event(CustomEvents::ShouldRedraw(*wid)).unwrap();
	}
}

impl FrameLimiter {
	pub fn new(event_loop: &EventLoop<CustomEvents>) -> Self {

		let (sender, receiver) = mpsc::channel::<WindowId>();
		let event_proxy = event_loop.create_proxy();

		thread::spawn(move || {

			let mut schedule = FrameSchedule::new(event_proxy);

			loop {
				match schedule.time_to_next_frame() {
					None => {
						let wid = receiver.recv().unwrap();
						schedule.insert(wid);
					}

					Some(dur) => {
						match receiver.recv_timeout(dur) {
							Ok(wid) => {
								schedule.insert(wid);
							}
							Err(mpsc::RecvTimeoutError::Timeout) => {
								schedule.process_due_frames();
							}
							Err(mpsc::RecvTimeoutError::Disconnected) => {
								return;
							}
						}
					}
				}
			}
		});

		FrameLimiter {
			sender,
		}
	}

	pub fn schedule_redraw(&self, wid: WindowId) {
		self.sender.send(wid).unwrap();
	}
}

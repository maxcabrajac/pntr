use bytemuck::{Pod, Zeroable};
use core::ops;

use std::{any::TypeId, collections::HashMap, sync::{Arc, Weak}};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Size {
	pub w: u32,
	pub h: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Rect {
	pub pos: Point,
	pub size: Size,
}

impl Rect {
	fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
		Self {
			pos: Point { x, y },
			size: Size { w, h },
		}
	}

	fn inside(&self, point: Point) -> bool {
		macro_rules! inside_dim {
			($dimP:ident, $dimS:ident) => {
				self.pos.$dimP <= point.$dimP
					&& point.$dimP <= self.pos.$dimP + self.size.$dimS as i32
			};
		}
		inside_dim!(x, w) && inside_dim!(y, h)
	}
}

impl ops::AddAssign for Point {
	fn add_assign(&mut self, other: Self) {
		self.x += other.x;
		self.y += other.y;
	}
}

impl ops::Add for Point {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		let mut r = self.clone();
		r += other;
		r
	}
}

impl ops::AddAssign<Point> for Rect {
	fn add_assign(&mut self, other: Point) {
		self.pos += other;
	}
}

impl ops::Add<Point> for Rect {
	type Output = Self;

	fn add(self, other: Point) -> Self::Output {
		let mut r = self.clone();
		r += other;
		r
	}
}

pub struct Pipelines {
	pub render: Vec<wgpu::RenderPipeline>,
	pub compute: Vec<wgpu::ComputePipeline>,
}

pub trait Component {
	fn generate_pipelines(_: &Context) -> Pipelines;
	fn new(_: &mut Context) -> Box<Self>;
	fn set_rect() -> Result<(), ()>;
	fn min_size() -> Option<Size>;
	fn render(
		&self,
		_: &mut wgpu::CommandEncoder,
		_: &Context,
		_: &wgpu::TextureView,
		_: Size,
	);
}

pub struct Context {
	pub device: wgpu::Device,
	pub surface_format: wgpu::TextureFormat,
	pipeline_map: HashMap<TypeId, Weak<Pipelines>>,
}

impl Context {
	pub fn new(device: wgpu::Device, surface_format: wgpu::TextureFormat) -> Context {
		Context {
			device,
			surface_format,
			pipeline_map: HashMap::new(),
		}
	}

	pub fn get_pipelines<T: Component + 'static>(&mut self) -> Arc<Pipelines> {
		if let Some(weak) = self.pipeline_map.get(&TypeId::of::<T>()) {
			if let Some(arc) = weak.upgrade() {
				return arc;
			}
		}

		let arc = Arc::new(T::generate_pipelines(self));
		self.pipeline_map.insert(TypeId::of::<T>(), Arc::<>::downgrade(&arc));
		return arc;
	}
}

macro_rules! add_component {
	($x:ident) => {
		mod $x;
		pub use crate::components::$x::*;
	};
}

add_component!(canvas);
add_component!(image);
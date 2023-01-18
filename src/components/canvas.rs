use std::collections::VecDeque;

use crate::components::{self, Point, Rect, Size, Image, Context, Pipelines};

// TODO: Use renderBundle in conjunction with buffers to draw different lines in the canvas without reencoding the render pass.

const BACKGROUND_COLOR: [f32; 3] = [0., 0., 0.];
const BRUSH_RADIUS: u32 = 3;
const TEX_SIZE: Size = Size { w: 2000, h: 2000 };

const POINTS_PER_BUFF: usize = 100;
const BUFF_SIZE: wgpu::BufferSize = match wgpu::BufferSize::new((POINTS_PER_BUFF * std::mem::size_of::<Point>()) as u64) {
	None => panic!("Error on BUFF_SIZE const definition"),
	Some(x) => x,
};

pub struct Canvas {
	pipelines: std::sync::Arc<Pipelines>,
	image: Box<Image>,
	tex_size: Size,
	brush_radius: u32,
	backgroud: [f32; 3],

	line_buff: wgpu::Buffer,
	line_binding: wgpu::BindGroup,

	line_points: VecDeque<VecDeque<Point>>,
	mouse_pos: Option<Point>,
	mouse_down: bool,
	clear: bool,
}

impl components::Component for Canvas {
	fn generate_pipelines(ctx: &Context) -> Pipelines {
		let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/canvas.wgsl"));

		let binding_group_layout = ctx.device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("Canvas(Binding Group Layout)"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::StorageTexture {
							access: wgpu::StorageTextureAccess::ReadWrite,
							format: wgpu::TextureFormat::Rgba8Unorm,
							view_dimension: wgpu::TextureViewDimension::D2
						},
						count: None,
					}
				]
			}
		);

		let line_list_layout = ctx.device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("Canvas(Line List Layout)"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Storage {
								read_only: true,
							},
							has_dynamic_offset: false,
							min_binding_size: core::num::NonZeroU64::new(4*2),

						},
						count: None,
					}
				]
			}
		);

		let point_pipeline_layout = ctx.device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Canvas(Point Pipeline Layout)"),
				bind_group_layouts: &[&binding_group_layout],
				push_constant_ranges: &[
					wgpu::PushConstantRange {
						stages: wgpu::ShaderStages::COMPUTE,
						range: (0..12),
					}
				],
			}
		);

		let point_pipeline = ctx.device.create_compute_pipeline(
			&wgpu::ComputePipelineDescriptor {
				label: Some("Canvas(Point Pipeline)"),
				layout: Some(&point_pipeline_layout),
				module: &shader,
				entry_point: "draw_point",
			}
		);

		let line_pipeline_layout = ctx.device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Canvas(Line Pipeline Layout)"),
				bind_group_layouts: &[&binding_group_layout, &line_list_layout],
				push_constant_ranges: &[
					wgpu::PushConstantRange {
						stages: wgpu::ShaderStages::COMPUTE,
						range: (0..9*4),
					}
				],
			}
		);

		let line_pipeline = ctx.device.create_compute_pipeline(
			&wgpu::ComputePipelineDescriptor {
				label: Some("Canvas(Line Pipeline)"),
				layout: Some(&line_pipeline_layout),
				module: &shader,
				entry_point: "draw_line",
			}
		);

		let clear_pipeline_layout = ctx.device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Canvas(Compute Pipeline Layout)"),
				bind_group_layouts: &[&binding_group_layout],
				push_constant_ranges: &[
					wgpu::PushConstantRange {
						stages: wgpu::ShaderStages::COMPUTE,
						range: (0..12),
					}
				],
			}
		);

		let clear_pipeline = ctx.device.create_compute_pipeline(
			&wgpu::ComputePipelineDescriptor {
				label: Some("Canvas(Compute Pipeline)"),
				layout: Some(&clear_pipeline_layout),
				module: &shader,
				entry_point: "clear",
			}
		);

		return Pipelines {
			render: vec![],
			compute: vec![clear_pipeline, point_pipeline, line_pipeline],
		};
	}
	fn new(ctx: &mut Context) -> Box<Self> {
		let tex_size = TEX_SIZE;
		let tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Canvas(Texture)"),
			size: wgpu::Extent3d {
				width: tex_size.w,
				height: tex_size.h,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::STORAGE_BINDING,
		});

		let pipelines = ctx.get_pipelines::<Self>();

		let line_buff = ctx.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Canvas(Line Buffer)"),
			size: BUFF_SIZE.into(),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let line_binding = ctx.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("Canvas(Binding group 1)"),
				layout: &pipelines.compute[2].get_bind_group_layout(1),
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: line_buff.as_entire_binding(),
					}
				],
			}
		);

		let mut image = Image::new(ctx);
		image.set_texture(ctx, tex);

		Box::new(Self {
			pipelines,
			image,
			tex_size,

			line_buff,
			line_binding,

			brush_radius: BRUSH_RADIUS,
			backgroud: BACKGROUND_COLOR,
			line_points: VecDeque::new(),
			mouse_pos: None,
			mouse_down: false,
			clear: true,
		})
	}

	fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut Context, output: &wgpu::TextureView, viewport: Rect, _clip_space: Option<Rect>) {

		let tex_view = self.image
			.get_texture()
			.as_ref()
			.unwrap()
			.create_view(&wgpu::TextureViewDescriptor::default());

		let binding_group = ctx.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("Canvas(Binding group 0)"),
				layout: &self.pipelines.compute[0].get_bind_group_layout(0),
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&tex_view),
					},
				],
			}
		);


		if self.clear {
			self.clear = false;
			let mut clear_pass = encoder.begin_compute_pass(
				&wgpu::ComputePassDescriptor {
					label: Some("Canvas(Clear Pass)"),
				}
			);

			clear_pass.set_pipeline(&self.pipelines.compute[0]);
			clear_pass.set_bind_group(0, &binding_group, &[]);
			clear_pass.set_push_constants(0, bytemuck::cast_slice(&self.backgroud));
			clear_pass.dispatch_workgroups((self.tex_size.w/8)+1, (self.tex_size.h/8)+1, 1);
		}

		if self.line_points.len() > 0 && self.line_points[0].len() > 1 {

			// Lines that ended
			let mut points_computed = 0;
			let mut i = 0;

			let mut mapped = ctx.staging_belt.write_buffer(encoder, &self.line_buff, 0, BUFF_SIZE, &ctx.device);

			let mut bundles: VecDeque<(Rect, u32, u32)> = VecDeque::new();

			let mut min_point: Point = self.line_points[0][0];
			let mut max_point: Point = min_point.clone();

			while points_computed < POINTS_PER_BUFF && i < self.line_points.len() {
				use std::cmp::{min, max};

				let mut bundle: (Rect, u32, u32) = (Rect::new(0, 0, 0, 0), 0, 0);

				if self.line_points[i].len() <= 1 {
					// Not a viable line
					break;
				}

				bundle.1 = points_computed as u32;

				for k in 0..min(POINTS_PER_BUFF - points_computed, self.line_points[i].len()) {
					let p = &self.line_points[i][k];

					const P_SIZE: usize = std::mem::size_of::<Point>();

					mapped[points_computed*P_SIZE..(points_computed+1)*P_SIZE].copy_from_slice(bytemuck::bytes_of(p));
					points_computed += 1;

					min_point.x = min(min_point.x, p.x);
					min_point.y = min(min_point.y, p.y);

					max_point.x = max(max_point.x, p.x);
					max_point.y = max(max_point.y, p.y);
				}

				let size = max_point - min_point;

				bundle.0 = Rect { pos: min_point, size: size.try_into().unwrap() };
				bundle.2 = points_computed as u32;

				bundles.push_back(bundle);

				if points_computed == POINTS_PER_BUFF {
					let f = file!();
					eprintln!("Buffer filled on this frame. If there was aditional lines to draw they were postponed to the next frame -> Should increase FPS or {f}::POINTS_PER_BUFF");
					break;
				}

				i += 1;
			}

			drop(mapped);

			let mut compute_pass = encoder.begin_compute_pass(
				&wgpu::ComputePassDescriptor {
					label: Some("Canvas(Compute Pass)"),
				}
			);

			compute_pass.set_pipeline(&self.pipelines.compute[2]);
			compute_pass.set_bind_group(0, &binding_group, &[]);
			compute_pass.set_bind_group(1, &self.line_binding, &[]);
			compute_pass.set_push_constants(4*4, bytemuck::bytes_of(&self.brush_radius));


			while !bundles.is_empty() {
				let reference = bundles[0].0.pos - Point {x: self.brush_radius as i32, y: self.brush_radius as i32};

				compute_pass.set_push_constants(0, bytemuck::bytes_of(&reference));
				compute_pass.set_push_constants(4*2, bytemuck::bytes_of(&bundles[0].1));
				compute_pass.set_push_constants(4*3, bytemuck::bytes_of(&bundles[0].2));

				let mut drawing_area = bundles[0].0.size.clone();
				drawing_area.w += 2*self.brush_radius;
				drawing_area.h += 2*self.brush_radius;


				compute_pass.dispatch_workgroups(drawing_area.w/8 + 1, drawing_area.h/8 + 1, 1);

				let mut to_be_removed = bundles[0].2 - bundles[0].1;

				if self.line_points.len() == 1 && self.mouse_down {
					to_be_removed -= 1;
				}

				self.line_points[0].drain(0..(to_be_removed.try_into().unwrap()));

				if self.line_points[0].is_empty() {
					self.line_points.pop_front();
				}

				bundles.pop_front();
			}
		}


		self.image.render(encoder, ctx, output, Rect::new(0, 0, self.tex_size.w, self.tex_size.h), Some(viewport));
	}

	fn min_size() -> Option<components::Size> {
		todo!()
	}
}

impl Canvas {
	pub fn mouse_pos(&mut self, p: Point) {
		if self.mouse_down && !self.line_points.is_empty() {
			self.line_points.back_mut().unwrap().push_back(self.mouse_pos.unwrap());
		}
		self.mouse_pos = Some(p);
	}

	pub fn mouse_up(&mut self) {
		self.mouse_down = false;
		if !self.line_points.is_empty() {
			self.line_points.back_mut().unwrap().push_back(self.mouse_pos.unwrap());
		}
	}

	pub fn mouse_down(&mut self) {
		self.mouse_down = true;
		self.line_points.push_back(VecDeque::new());
		self.line_points.back_mut().unwrap().push_back(self.mouse_pos.unwrap());
	}

	pub fn clear(&mut self) {
		self.clear = true;
	}
}

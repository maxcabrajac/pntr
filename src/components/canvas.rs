use crate::components::{self, Point, Rect, Size, Image, Context, Pipelines};

// TODO: Use renderBundle in conjunction with buffers to draw different lines in the canvas without reencoding the render pass.

const BACKGROUND_COLOR: [f32; 3] = [0., 0., 0.];
const BRUSH_RADIUS: u32 = 10;
const TEX_SIZE: Size = Size { w: 2000, h: 2000 };

pub struct Canvas {
	pipelines: std::sync::Arc<Pipelines>,
	image: Box<Image>,
	tex_size: Size,
	brush_radius: u32,
	backgroud: [f32; 3],

	mouse_pos: Option<Point>,
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

		let draw_pipeline_layout = ctx.device.create_pipeline_layout(
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

		let draw_pipeline = ctx.device.create_compute_pipeline(
			&wgpu::ComputePipelineDescriptor {
				label: Some("Canvas(Compute Pipeline)"),
				layout: Some(&draw_pipeline_layout),
				module: &shader,
				entry_point: "draw",
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
			compute: vec![draw_pipeline, clear_pipeline],
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

		let mut image = Image::new(ctx);
		image.set_texture(ctx, tex);

		Box::new(Self {
			pipelines: ctx.get_pipelines::<Self>(),
			image,
			tex_size,
			brush_radius: BRUSH_RADIUS,
			backgroud: BACKGROUND_COLOR,
			mouse_pos: None,
			clear: true,
		})
	}

	fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &Context, output: &wgpu::TextureView, viewport: Rect, clip_space: Option<Rect>) {

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

			clear_pass.set_pipeline(&self.pipelines.compute[1]);
			clear_pass.set_bind_group(0, &binding_group, &[]);
			clear_pass.set_push_constants(0, bytemuck::cast_slice(&self.backgroud));
			clear_pass.dispatch_workgroups((self.tex_size.w/8)+1, (self.tex_size.h/8)+1, 1);
		}

		let mut compute_pass = encoder.begin_compute_pass(
			&wgpu::ComputePassDescriptor {
				label: Some("Canvas(Compute Pass)"),
			}
		);

		compute_pass.set_pipeline(&self.pipelines.compute[0]);
		compute_pass.set_bind_group(0, &binding_group, &[]);
		let pc = [
			match self.mouse_pos {
				Some(p) => p,
				None => Point {x: 0, y: 0},
			},
		];

		let pc2 = [self.brush_radius];
		compute_pass.set_push_constants(0, bytemuck::cast_slice(&pc));
		compute_pass.set_push_constants(8, bytemuck::cast_slice(&pc2));

		let brush_workgroups = (self.brush_radius * 2 / 8) + 1;
		compute_pass.dispatch_workgroups(brush_workgroups, brush_workgroups, 1);

		drop(compute_pass);

		self.image.render(encoder, ctx, output, Rect::new(0, 0, self.tex_size.w, self.tex_size.h), Some(viewport));
	}

	fn min_size() -> Option<components::Size> {
		todo!()
	}
}

impl Canvas {
	pub fn mouse_pos(&mut self, p: Point) {
		dbg!(p);
		self.mouse_pos = Some(p);
	}

	pub fn mouse_leave(&mut self) {
		self.mouse_pos = None;
	}

	pub fn clear(&mut self) {
		self.clear = true;
	}
}

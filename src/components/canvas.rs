use crate::components::{self, Rect, Size, Image, Context, Pipelines};

// TODO: Use renderBundle in conjunction with buffers to draw different lines in the canvas without reencoding the render pass.

pub struct Canvas {
	pipelines: std::sync::Arc<Pipelines>,
	image: Box<Image>,
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

		let compute_pipeline_layout = ctx.device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Canvas(Compute Pipeline Layout)"),
				bind_group_layouts: &[&binding_group_layout],
				push_constant_ranges: &[],
			}
		);

		let compute_pipeline = ctx.device.create_compute_pipeline(
			&wgpu::ComputePipelineDescriptor {
				label: Some("Canvas(Compute Pipeline)"),
				layout: Some(&compute_pipeline_layout),
				module: &shader,
				entry_point: "cpt_main",
			}
		);

		return Pipelines {
			render: vec![],
			compute: vec![compute_pipeline],
		};
	}
	fn new(ctx: &mut Context) -> Box<Self> {

		let tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Canvas(Texture)"),
			size: wgpu::Extent3d {
				width: 2000,
				height: 2000,
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

		Box::new(Self{
			pipelines: ctx.get_pipelines::<Self>(),
			image
		})
	}

	fn render(&self, encoder: &mut wgpu::CommandEncoder, ctx: &Context, output: &wgpu::TextureView, viewport: Rect, clip_space: Option<Rect>) {

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

		let mut compute_pass = encoder.begin_compute_pass(
			&wgpu::ComputePassDescriptor {
				label: Some("Canvas(Compute Pass)"),
			}
		);

		compute_pass.set_pipeline(&self.pipelines.compute[0]);
		compute_pass.set_bind_group(0, &binding_group, &[]);
		compute_pass.dispatch_workgroups(200, 200, 1);

		drop(compute_pass);

		self.image.render(encoder, ctx, output, viewport, clip_space);

	}

	fn min_size() -> Option<components::Size> {
		todo!()
	}
}

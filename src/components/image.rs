use crate::components::{self, Rect, Context, Pipelines, RectViewportClipSpace};

pub struct Image {
	pipelines: std::sync::Arc<Pipelines>,
	tex: Option<wgpu::Texture>,
	binding_group: Option<wgpu::BindGroup>,
}

impl components::Component for Image {
	fn generate_pipelines(ctx: &Context) -> Pipelines {
		let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/image.wgsl"));

		let binding_group_layout = ctx.device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("Image(Binding Group Layout)"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::StorageTexture {
							access: wgpu::StorageTextureAccess::ReadOnly,
							format: wgpu::TextureFormat::Rgba8Unorm,
							view_dimension: wgpu::TextureViewDimension::D2
						},
						count: None,
					}
				]
			}
		);

		let render_pipeline_layout = ctx.device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Image(Pipeline Layout)"),
				bind_group_layouts: &[&binding_group_layout],

				push_constant_ranges: &[],
			}
		);


		let render_pipeline = ctx.device.create_render_pipeline(
			&wgpu::RenderPipelineDescriptor {
				label: Some("Image(Render Pipeline)"),
				layout: Some(&render_pipeline_layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[],
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: "fs_main",
					targets: &[Some(wgpu::ColorTargetState {
						format: ctx.surface_format,
						blend: Some(wgpu::BlendState::ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: None,
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
					conservative: false,
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState {
					count: 1,
					mask: !0,
					alpha_to_coverage_enabled: false
				},
				multiview: None
			}
		);

		Pipelines {
			render: vec![render_pipeline],
			compute: vec![],
		}
	}


	fn new(ctx: &mut Context) -> Box<Self> {
		Box::new(Self {
			pipelines: ctx.get_pipelines::<Self>(),
			tex: None,
			binding_group: None,
		})

	}

	fn render(&mut self, encoder: &mut wgpu::CommandEncoder, _: &mut Context, output: & wgpu::TextureView, viewport: Rect, clip_space: Option<Rect>) {
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Image(Render Pass)"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &output,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Load,
					store: true,
				}
			})],
			depth_stencil_attachment: None,
		});

		render_pass.set_pipeline(&self.pipelines.render[0]);
		render_pass.set_viewport_rect(viewport);
		render_pass.set_clipspace_rect(clip_space);
		let binding = self.binding_group.as_ref().expect("Trying to render Image with no texture");
		render_pass.set_bind_group(0, &binding, &[]);
		render_pass.draw(0..6, 0..1);

		drop(render_pass)
	}


	fn min_size() -> Option<components::Size> {
		todo!()
	}
}

impl Image {
	pub fn get_texture(&self) -> &Option<wgpu::Texture> {
		&self.tex
	}

	pub fn set_texture(&mut self, ctx: &Context, tex: wgpu::Texture) {
		self.tex = Some(tex);

		let tex_view = self.tex
			.as_ref()
			.unwrap()
			.create_view(&wgpu::TextureViewDescriptor::default());

		let binding_group = ctx.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("Image(Binding group 0)"),
				layout: &self.pipelines.render[0].get_bind_group_layout(0),
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&tex_view),
					},
				],
			}
		);

		self.binding_group = Some(binding_group);
	}
}

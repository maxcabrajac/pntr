use std::sync::Arc;
use winit::{
	event::WindowEvent,
	event_loop::EventLoopWindowTarget,
	window::Window,
};
use async_trait::async_trait;

pub enum WindowLifeStatus {
	Alive,
	Dead,
}

#[async_trait]
pub trait Layout {
	async fn new(_: Arc<Window>) -> Box<Self>
	where
		Self: Sized;
	fn window(&self) -> Arc<Window>;
	fn render(&mut self);
	fn update(
		&mut self,
		_: &EventLoopWindowTarget<()>,
	) -> (WindowLifeStatus, Option<Box<dyn Layout>>);

	/// Returns a life status and maybe another layout, notice that if the child layout uses the same window, the parent layout must pronounce itself as dead.
	fn event_handler(&mut self, _: winit::event::WindowEvent);
}

pub struct Triangle {
	window: Arc<Window>,
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: winit::dpi::PhysicalSize<u32>,

	pipeline: wgpu::RenderPipeline,
	cpipeline: wgpu::ComputePipeline,

	//Events:
	resized: bool,
	close: bool,
}

#[async_trait]
impl Layout for Triangle {
	async fn new(window: Arc<Window>) -> Box<Self> {
		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(window.as_ref()) };


		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.expect("Could not get adapter");

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
					limits: wgpu::Limits::default(),
					label: None,
				},
				None,
			)
			.await
			.expect("Could not get device-queue pair");

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_supported_formats(&adapter)[0],
			width: size.width,
			height: size.height,
			// TODO: Try AutoNoVsync
			present_mode: wgpu::PresentMode::AutoVsync,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
		};

		surface.configure(&device, &config);

		let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
			label: Some("Bind group layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::StorageTexture {
						access: wgpu::StorageTextureAccess::ReadWrite,
						format: wgpu::TextureFormat::Rgba8Unorm,
						view_dimension: wgpu::TextureViewDimension::D2
					},
					count: None,
				}
			],
		});


		let render_pipeline_layout =
		device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[],
			},
			fragment: Some(wgpu::FragmentState{
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				})]
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
			multiview: None,
		});

		let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let cpipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("Compute Pipeline"),
			layout: Some(&compute_pipeline_layout),
			module: &shader,
			entry_point: "compute_main",
		});

		return Box::new(Triangle {
			window,
			surface,
			device,
			queue,
			config,
			size,

			pipeline,
			cpipeline,

			resized: false,
			close: false,
		});
	}

	fn window(&self) -> Arc<Window> {
		self.window.clone()
	}

	fn render(&mut self) {
		println!("rendering");
		match self.surface.get_current_texture() {
			Err(wgpu::SurfaceError::Lost) => self.resized = true,
			Err(wgpu::SurfaceError::OutOfMemory) => self.close = true,
			Err(e) => eprintln!("{:?}", e),
			Ok(output) => {
				let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

				let tex = self.device.create_texture(&wgpu::TextureDescriptor{
					label: Some("Texture"),
					size: wgpu::Extent3d{
						width: 2000,
						height: 2000,
						depth_or_array_layers: 1
					},
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Rgba8Unorm,
					usage: wgpu::TextureUsages::STORAGE_BINDING,
				});

				let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());

				let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label: Some("Render Encoder"),
				});


				let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor{
					label: Some("Compute Bind Group"),
					layout: &self.cpipeline.get_bind_group_layout(0),
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&tex_view),
						}
					]
				});

				let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
					label: Some("Compute Pass"),
				});

				compute_pass.set_pipeline(&self.cpipeline);
				compute_pass.set_bind_group(0, &compute_bind_group, &[]);
				compute_pass.dispatch_workgroups(self.size.width, self.size.height, 1);

				drop(compute_pass);

				let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
					label: Some("Render Pass"),
					color_attachments: &[Some(wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 1.0,
							}),
							store: true,
						}
					})],
					depth_stencil_attachment: None,
				});

				render_pass.set_pipeline(&self.pipeline);
				render_pass.set_bind_group(0, &compute_bind_group, &[]);
				render_pass.draw(0..3, 0..1);

				drop(render_pass);

				self.queue.submit(std::iter::once(encoder.finish()));
				output.present();
			}
		}

	}

	fn update(
		&mut self,
		_: &EventLoopWindowTarget<()>,
	) -> (WindowLifeStatus, Option<Box<dyn Layout>>) {
		use WindowLifeStatus::*;

		if self.resized {
			self.resized = false;
			let new_size = self.window().inner_size();
			if new_size.width <= 0 || new_size.height <= 0 {
				return (Alive, None)
			}

			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}

		if self.close {
			self.close = false;
			return (Dead, None);
		}


		(Alive, None)
	}

	fn event_handler(&mut self, event: winit::event::WindowEvent) {
		use WindowEvent::*;
		match event {
			CloseRequested => self.close = true,
			Resized(_) | WindowEvent::ScaleFactorChanged {..} => {
				self.resized = true;
			},
			KeyboardInput {
				input: winit::event::KeyboardInput {
					state: winit::event::ElementState::Released,
					virtual_keycode: Some(letter),
					..
				},
				..
			} => {
				match letter {
					_ => ()
				}
			},
			_ => (),
		}
	}
}

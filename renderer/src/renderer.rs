use image::GenericImageView;
use wgpu::util::DeviceExt;
use winit::{
	event::WindowEvent,
	window::Window,
};

pub struct RendererState {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub surface_config: wgpu::SurfaceConfiguration,
	pub size: winit::dpi::PhysicalSize<u32>,
	pub render_pipeline: wgpu::RenderPipeline,
	pub vertex_buffer: wgpu::Buffer,
	pub num_vertices: u32,
	pub diffuse_bind_group: wgpu::BindGroup,
	diffuse_texture: crate::textures::Texture,
	pub window: Window,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
	Vertex { position: [-0.4, 0.4, 0.0], tex_coords: [0.0, 0.0] },
	Vertex { position: [0.4, 0.4, 0.0], tex_coords: [1.0, 0.0] },
	Vertex { position: [0.4, -0.4, 0.0], tex_coords: [1.0, 1.0] },
	Vertex { position: [-0.4, 0.4, 0.0], tex_coords: [0.0, 0.0] },
	Vertex { position: [0.4, -0.4, 0.0], tex_coords: [1.0, 1.0] },
	Vertex { position: [-0.4, -0.4, 0.0], tex_coords: [0.0, 1.0] },
];

impl Vertex {
	fn desc() -> wgpu::VertexBufferLayout<'static> {
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x2,
				},
			],
		}
	}
}

impl RendererState {
	pub async fn new(window: Window) -> Self {
		let size = window.inner_size();

		// Create instance and surface.
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			dx12_shader_compiler: Default::default(),
		});
		let surface = unsafe { instance.create_surface(&window) }.unwrap();

		// Create adapter
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptionsBase {
				power_preference: wgpu::PowerPreference::HighPerformance,
				force_fallback_adapter: false,
				compatible_surface: Some(&surface),
			})
			.await
			.unwrap();

		// Create device and queue
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::default(),
					label: None,
				},
				None,
			)
			.await
			.unwrap();

		let surface_caps = surface.get_capabilities(&adapter);
		let surface_format = surface_caps
			.formats
			.iter()
			.copied()
			.find(|f| f.is_srgb())
			.unwrap_or(surface_caps.formats[0]);

		let surface_config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &surface_config);

		let diffuse_bytes = include_bytes!("../../resources/white.png");
		let diffuse_texture =
			crate::textures::Texture::from_bytes(&device, &queue, diffuse_bytes, "white.png")
				.unwrap();

		let texture_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: Some("texture_bind_group_layout"),
			});
		let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
				},
			],
			label: Some("diffuse_bind_group"),
		});

		let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("render_pipeline_layout"),
				bind_group_layouts: &[&texture_bind_group_layout],
				push_constant_ranges: &[],
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("render_pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[Vertex::desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Front),
				unclipped_depth: false,
				polygon_mode: wgpu::PolygonMode::Fill,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("vertex_buffer"),
			contents: bytemuck::cast_slice(VERTICES),
			usage: wgpu::BufferUsages::VERTEX,
		});

		let num_vertices = VERTICES.len() as u32;

		Self {
			window,
			surface,
			device,
			queue,
			surface_config,
			size,
			render_pipeline,
			vertex_buffer,
			num_vertices,
			diffuse_bind_group,
			diffuse_texture,
		}
	}

	pub fn window(&self) -> &Window {
		&self.window
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.surface_config.width = new_size.width;
			self.surface_config.height = new_size.height;
			self.surface.configure(&self.device, &self.surface_config);
		}
	}

	pub fn input(&mut self, event: &WindowEvent) -> bool {
		false
	}

	pub fn update(&mut self) {}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("render_encoder"),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("render_pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.01298,
							g: 0.01298,
							b: 0.02732,
							a: 1.0,
						}),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.draw(0..self.num_vertices, 0..1);
		}

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}

mod texture;
mod camera;

use texture::Texture;

use winit::{event::{WindowEvent,VirtualKeyCode,ElementState}, window::{Window, WindowBuilder}};



use wgpu::util::DeviceExt;


//state of rendering
//this is thicc
pub struct State 
{
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    sc: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    bgcolor: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    camera: camera::Camera,
    uniforms: camera::Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    camera_controller: camera::CameraController,
}


//vertex data
//represents? a c struct that stores an 2 arrays of size 3 that hold f32 
#[repr(C)]
#[derive(Copy,Clone,Debug,bytemuck::Pod,bytemuck::Zeroable)]
struct Vertex
{
    position: [f32; 3],
    tex_coords: [f32; 2],
}


// Changed
//this is the vertecies that get passed to gpu and are drawn
const VERTICIES: &[Vertex] = &[
    Vertex { position: [-0.0868241+10f32, 0.49240386, 0.0], tex_coords: [0.4131759+10f32, 1.0-0.99240386], }, // A
    Vertex { position: [-0.49513406+10f32, 0.06958647, 0.0], tex_coords: [0.0048659444+10f32, 1.0-0.56958646], }, // B
    Vertex { position: [-0.21918549+10f32, -0.44939706, 0.0], tex_coords: [0.28081453+10f32, 1.0-0.050602943], }, // C
    Vertex { position: [0.35966998+10f32, -0.3473291, 0.0], tex_coords: [0.85967+10f32, 1.0-0.15267089], }, // D
    Vertex { position: [0.44147372+10f32, 0.2347359, 0.0], tex_coords: [0.9414737+10f32, 1.0-0.7347359], }, // E
];


const INDICES: &[u16] = &[
    0,1,4,
    1,2,4,
    2,3,4,
];

//functions for State struct
impl State 
{
    //creating some wgpu types requires async code
    pub async fn new(window: &Window) -> Self 
    {
        let size = window.inner_size();

        //instance is handle to our GPU
        //BackendBit::PRIMARY => Vulkan + Metal + DX12 + WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        //used in the swapchain, also needed to request adapter
        let surface = unsafe {instance.create_surface(window)};
        //used to create device and queue
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions
            { 
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();
        

        //this returns both a devices and a queue for the device. 
        //devices have a lot of features being unused, worth exploring. some features limit compatibility.
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            }, None,//trace_path)
        ).await.unwrap();

        device.features();

        let sc_desc = wgpu::SwapChainDescriptor 
        {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        let sc = device.create_swap_chain(&surface, &sc_desc);

        let diffuse_bytes = include_bytes!("baseparticle.bmp");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.as_rgba8().unwrap();

        use image::GenericImageView;

        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d
        {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        //this is empty upon init
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "baseparticle.bmp").unwrap();

        let bgcolor = wgpu::Color
        {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        //bind group describes a set of resources and how they can be accessed by a shader
        let texture_bind_group_layout =device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor
            {
                entries: &[
                    wgpu::BindGroupLayoutEntry
                    {
                        binding:0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture
                        {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry
                    {
                        binding:1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler
                        {
                            comparison:false,
                        },
                        count: None,
                    },
  
                ],
                label: Some("texture_bind_group_layout"),
            }
          );
  
          let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor
            {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry
                    {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry
                    {
                        binding:1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }  
          );

          let camera = camera::Camera 
          {
              eye: (0.0,1.0,2.0).into(),
              target:(0.0,0.0,0.0).into(),
              up: cgmath::Vector3::unit_y(),
              aspect: sc_desc.width as f32 / sc_desc.height as f32,
              zoom: 10.0,
              fovy: 90.0,
              znear: 0.01,
              zfar: 1000.0,
          };

          let mut uniforms = camera::Uniforms::new(); 
          uniforms.update_view_proj(&camera);

          let uniform_buffer = device.create_buffer_init(
              &wgpu::util::BufferInitDescriptor
              {
                  label: Some("Uniform Buffer"),
                  contents: bytemuck::cast_slice(&[uniforms]),
                  usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
              }
          );
          
          let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });
         
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..))
                }
            ],
            label: Some("uniform_bind_group"),
        });
         






        //let vs_src = include_str!("shader.vert");
        //let fs_src = include_str!("shader.frag");

       //let mut complier = shaderc::Compiler::new().unwrap();

        //let vs_spirv = complier.compile_into_spirv(&vs_src, shaderc::ShaderKind::Vertex, "shader.vert","main", None).unwrap(); 
       // let fs_spirv = complier.compile_into_spirv(&fs_src, shaderc::ShaderKind::Fragment,"shader.frag","main",None).unwrap();

        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));



        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor 
            {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICIES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor
            {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );


        let index_count = INDICES.len() as u32;
        
        let render_pipeline = create_pipeline(
            &device, 
            &sc_desc, 
            vs_module, 
            fs_module, 
            texture_bind_group_layout,
        uniform_bind_group_layout);

        let camera_controller = camera::CameraController::new(0.002);
        Self
        {
            surface,
            device,
            queue,
            sc_desc,
            sc,
            size,
            last_fps_text_change: None,
            fps_text: "".to_string(),
            bgcolor,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            camera_controller,
        }

    }
    //end of async code

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.sc = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool 
    {
        self.camera_controller.process_events(event);
        match &event
        {
            WindowEvent::CursorMoved{position, ..} =>
            {
                let color = wgpu::Color
                {
                    r: position.x.sin(),
                    g: position.y.cos(),
                    b: position.y.sin(),
                    a: 1.0,
                };

                self.bgcolor = color;
                true
            }
            _ => {false}
        }

    }

    pub fn update(&mut self)
    {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> 
    {
        let frame = self
        .sc
        .get_current_frame()?
        .output;

        //command encoder builds command buffer that can be sent to gpu
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor
        {
            label: Some("Render Encoder"),
        });

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
            {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor
                    {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations
                        {
                            load: wgpu::LoadOp::Clear(self.bgcolor),
                            store: true,
                        }
                    }
                ],
                depth_stencil_attachment: None,
            });


            _render_pass.set_pipeline(&self.render_pipeline);
            _render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            _render_pass.set_bind_group(1,   &self.uniform_bind_group, &[]);
            _render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _render_pass.set_index_buffer(self.index_buffer.slice(..));
            _render_pass.draw_indexed(0..self.index_count,0, 0..1);
        }

        //submit accepts anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

}


impl Vertex 
{
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>
    {
        wgpu::VertexBufferDescriptor
        {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor
                {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor
                {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,

                }
            ]

        }
    }
}

fn mainfunction() 
{
    

}
 

fn create_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout) -> wgpu::RenderPipeline
{
    let render_pipeline_layout = 
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout,
            &uniform_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
{
    label: Some("Render Pipeline"),
    layout: Some(&render_pipeline_layout),
    vertex_stage: wgpu::ProgrammableStageDescriptor
    {
        module: &vs_module,
        entry_point: "main", //1 
    },
    fragment_stage: Some(wgpu::ProgrammableStageDescriptor
    {
        module: &fs_module,
        entry_point: "main",//2 
    }),
    //continued.... im guesssing pipeline can have any number of shaders 
    //notes
    //doesnt have to be called main, could be any function in the shader.


    //describes how to process primatives, triangles in this case, before theyre sent to 
    //fragment shader (or next stage in pipeline if theres none)
    //primatives that dont meet the criteria are culled, for faster rendering.
    rasterization_state: Some(
        wgpu::RasterizationStateDescriptor
        {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }
    ),

    
    //color states describes how colors are stored and processed through the pipeline. can have multiple
    //we use the swap chain format so copying is ez
    //specified that the new pixels just replace the old pixels
    color_states: &[
        wgpu::ColorStateDescriptor
        {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        },
    ],

    primitive_topology: wgpu::PrimitiveTopology::TriangleList, 
    depth_stencil_state: None,
    vertex_state: wgpu:: VertexStateDescriptor
    {
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[Vertex::desc(),
        ],
    },
    sample_count: 1,
    sample_mask: !0,
    alpha_to_coverage_enabled: false,

});

render_pipeline
}

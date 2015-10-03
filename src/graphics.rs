
use regl::{Context,Shader,ShaderSource,ShaderType,Program,
    VertexArray,Buffer,BufferTarget,BufferUsage,RenderOption,
    VertexAttribute,VertexAttributeType,PrimitiveMode,IndexType};

#[allow(dead_code)]
#[repr(C,packed)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[allow(dead_code)]
#[repr(C,packed)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[allow(dead_code)]
#[repr(C,packed)]
struct Vertex {
    position: Vec3,
    color: Rgba
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32, r: u8, g: u8, b: u8, a: u8) -> Vertex {
        Vertex { position: Vec3 { x: x, y: y, z: z }, color: Rgba { r: r, g: g, b: b, a: a } }
    }
}

static VS_SOURCE: &'static str = "
#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

uniform float scale;

out vec4 v_color;

void main() {
    gl_Position.xyz = position * scale;
    gl_Position.w = 1.0;
    v_color = color;
}
";

static FS_SOURCE: &'static str = "
#version 330 core

in vec4 v_color;
out vec3 color;

void main() {
    color = v_color.rgb;
}
";

pub struct Graphics {
    context: Context,
    vao: VertexArray,
    program: Program,
}

impl Graphics {
    pub fn new() -> Graphics {
        let mut context = ::regl::Context::new();

        context.set_option(RenderOption::CullingEnabled(false));
        context.default_framebuffer().clear_color(0.0, 0.0, 0.0, 1.0);

        let vertices = &[
            Vertex::new(-0.5f32, -0.5f32, 0f32, 255, 0, 0, 0),
            Vertex::new(0.5f32, -0.5f32, 0f32, 0, 255, 0, 0),
            Vertex::new(0f32, 0.5f32, 0f32, 0, 0, 255, 0),
        ];
        let vbo = Buffer::new(&mut context, BufferTarget::VertexBuffer, BufferUsage::StaticDraw, vertices).unwrap();
        let attributes = &[
            VertexAttribute {
                index: 0,
                size: 3,
                attribute_type: VertexAttributeType::Float,
                normalized: false,
                stride: 16,
                offset: 0,
                vertex_buffer: &vbo
            },
            VertexAttribute {
                index: 1,
                size: 4,
                attribute_type: VertexAttributeType::UnsignedByte,
                normalized: true,
                stride: 16,
                offset: 12,
                vertex_buffer: &vbo
            }
        ];
        let indices: &[u16] = &[0, 1, 2];
        let ibo = Buffer::new(&mut context, BufferTarget::IndexBuffer, BufferUsage::StaticDraw, indices).unwrap();
        let vao = VertexArray::new(&mut context, attributes, Some(&ibo)).unwrap();
        let vs = Shader::new(&mut context, &ShaderSource(ShaderType::VertexShader, VS_SOURCE)).unwrap();
        let fs = Shader::new(&mut context, &ShaderSource(ShaderType::FragmentShader, FS_SOURCE)).unwrap();
        let program = Program::new(&mut context, &[vs, fs]).unwrap();

        let uniform_info = program.uniform_info();
        let scale = uniform_info.find_global("scale").unwrap();
        program.uniform_f32(scale.location, scale.uniform_type, 1, &[1f32]).unwrap();

        //println!("{:#?}", program.attribute_info());
        //println!("{:#?}", uniform_info);

        Graphics {
            context: context,
            vao: vao,
            program: program,
        }
    }

    pub fn draw(&self) {
        self.context.default_framebuffer().clear();
        //context.draw(&program, context.default_framebuffer(), &vao, PrimitiveMode::Triangles, 0, 3);
        self.context.draw_indexed(
            &self.program,
            self.context.default_framebuffer(),
            &self.vao,
            PrimitiveMode::Triangles, IndexType::UShort, 0, 3, 0);
    }
}

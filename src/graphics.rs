
use ::cgmath::{Matrix4,Vector3,Point3,Basis3,Rotation3,Rotation,Rad};
use regl::{Context,Shader,ShaderSource,ShaderType,Program,
    VertexArray,Buffer,BufferTarget,BufferUsage,RenderOption,
    VertexAttribute,VertexAttributeType,PrimitiveMode,IndexType,UniformType};

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

const YAW_MULTIPLIER: f32 = 1.0 / 60.0;
const PITCH_MULTIPLIER: f32 = 1.0 / 60.0;
const DISTANCE_MULTIPLIER: f32 = 0.5;

const VS_SOURCE: &'static str = "
#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

uniform mat4 worldview;
uniform mat4 projection;

out vec4 v_color;

void main() {
    vec4 position = vec4(position, 1);
    position = worldview * position;
    position = projection * position;
    gl_Position = position; //(worldview * projection) * position;
    v_color = color;
}
";

const FS_SOURCE: &'static str = "
#version 330 core

in vec4 v_color;
out vec3 color;

void main() {
    color = v_color.rgb;
}
";

pub struct Graphics {
    viewport_size: (i32, i32),
    context: Context,
    vao: VertexArray,
    program: Program,
    camera_orientation: (f32, f32),
    camera_distance: f32,
    projection_uniform: i32,
    worldview_uniform: i32,
}

impl Graphics {
    pub fn new(width: i32, height: i32) -> Graphics {
        let mut context = ::regl::Context::new();

        context.set_option(RenderOption::CullingEnabled(false));
        context.default_framebuffer().clear_color(0.0, 0.0, 0.0, 1.0);

        let z = -0.0;
        let vertices = &[
            Vertex::new(-0.5f32, -0.5f32, z, 255, 0, 0, 0),
            Vertex::new(0.5f32, -0.5f32, z, 0, 255, 0, 0),
            Vertex::new(0f32, 0.5f32, z, 0, 0, 255, 0),
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

        let projection_location = program.uniform_location("projection").unwrap();
        let worldview_location = program.uniform_location("worldview").unwrap();

        Graphics {
            viewport_size: (width, height),
            context: context,
            vao: vao,
            program: program,
            camera_distance: 1.5,
            camera_orientation: (0.0, 0.0),
            projection_uniform: projection_location,
            worldview_uniform: worldview_location,
        }
    }

    pub fn camera_orientation(&mut self, delta_x: f64, delta_y: f64) {
        self.camera_orientation.0 += delta_x as f32;
        self.camera_orientation.1 += delta_y as f32;
        let max = ::std::f32::consts::FRAC_PI_2 / PITCH_MULTIPLIER * 0.95;
        let min = -max;
        self.camera_orientation.1 = self.camera_orientation.1.min(max).max(min);
    }

    pub fn camera_distance(&mut self, delta: f64) {
        self.camera_distance += delta as f32;
        let max = 15.0 / DISTANCE_MULTIPLIER;
        let min = 0.5 / DISTANCE_MULTIPLIER;
        self.camera_distance = self.camera_distance.min(max).max(min);
    }

    pub fn viewport_size(&mut self, width: i32, height: i32) {
        self.viewport_size = (width, height);
        self.context.viewport(0, 0, width, height);
    }

    pub fn draw(&self) {
        let fov = ::std::f32::consts::FRAC_PI_2 * 0.5;
        let aspect = self.viewport_size.0 as f32 / self.viewport_size.1 as f32;
        let projection = perspective(fov, aspect, 1.0, 10.0);
        self.program.uniform_f32(self.projection_uniform, UniformType::FloatMat4, 1, floats(&projection)).unwrap();

        let worldview = self.worldview();
        self.program.uniform_f32(self.worldview_uniform, UniformType::FloatMat4, 1, floats(&worldview)).unwrap();

        self.context.default_framebuffer().clear();
        self.context.draw_indexed(
            &self.program,
            self.context.default_framebuffer(),
            &self.vao,
            PrimitiveMode::Triangles, IndexType::UShort, 0, 3, 0);
    }

    fn worldview(&self) -> Matrix4<f32> {
        let scaled_distance = DISTANCE_MULTIPLIER * self.camera_distance;
        let yaw = self.camera_orientation.0 * YAW_MULTIPLIER;
        let pitch = self.camera_orientation.1 * PITCH_MULTIPLIER;

        let basis = Basis3::from_euler(rad(pitch), rad(yaw), rad(0.0));
        let eye_vec = basis.rotate_vector(&Vector3::new(0.0, 0.0, scaled_distance));
        let eye = Point3::new(eye_vec.x, eye_vec.y, eye_vec.z);

        let center = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::unit_y();
        Matrix4::look_at(&eye, &center, &up)
    }
}

fn perspective(fov_y: f32, aspect: f32, z_near: f32, z_far: f32) -> Matrix4<f32> {
    let f = cot(fov_y / 2.0);
    Matrix4::new(
        f / aspect, 0.0, 0.0,                               0.0,
        0.0,        f,   0.0,                               0.0,
        0.0,        0.0, (z_far + z_near)/(z_near - z_far), (2.0 * z_far * z_near)/(z_near - z_far),
        0.0,        0.0, -1.0,                              0.0,
    )
}

fn floats<T>(matrix: &Matrix4<T>) -> &[T; 16] {
    AsRef::as_ref(matrix)
}

fn cot(angle: f32) -> f32 {
    angle.tan().recip()
}

fn rad(value: f32) -> Rad<f32> {
    Rad { s: value }
}

extern crate gl;
extern crate glutin;

extern crate time;
extern crate rng;
extern crate pseudo_random;

#[macro_use]
extern crate vector;

extern crate material;
extern crate shader;

extern crate camera_components;
extern crate transform_components;

extern crate scene_graph;
extern crate scene_renderer;
extern crate gl_renderer_plugin;
extern crate sprite_component;
extern crate sprite_gl_renderer;


use rng::Rng;
use pseudo_random::Prng;

use material::Material;
use shader::Shader;

use camera_components::{Camera3D, Camera3DManager};
use transform_components::Transform3D;

use scene_graph::{Scene, Entity, Component};
use scene_renderer::SceneRenderer;
use gl_renderer_plugin::GLRendererPlugin;
use sprite_component::Sprite;
use sprite_gl_renderer::SpriteGLRenderer;


static VS_SRC: &'static str = "
    #version 140

    in vec3 position;
    in vec3 normal;
    in vec2 uv;

    uniform mat4 projection;
    uniform mat4 model_view;

    varying vec3 v_normal;
    varying vec2 v_uv;

    void main() {
        v_normal = normal;
        v_uv = uv;
        gl_Position = projection * model_view * vec4(position, 1.0);
    }
";
static FS_SRC: &'static str = "
    #version 140

    out vec4 out_color;

    varying vec3 v_normal;
    varying vec2 v_uv;

    void main() {
        out_color = vec4(v_uv, v_normal.z, 1.0);
    }
";


fn main() {
    let window = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .build()
        .unwrap();

    let mut scene = Scene::new();
    let mut scene_renderer = SceneRenderer::new(scene.clone());

    scene_renderer.add_plugin(GLRendererPlugin::new());
    scene_renderer.add_renderer(SpriteGLRenderer::new());


    {
        let mut entity = Entity::new();

        let mut transform = Transform3D::new();
        transform.set_position(&[10f32, 10f32, 5f32]);
        transform.look_at(&[0f32, 0f32, 0f32], &[0f32, 0f32, 1f32]);
        entity.add_component(transform);

        let mut camera3d = Camera3D::new();
        camera3d.set_background(&[0.3, 0.3, 0.3, 1.0]);
        camera3d.set_orthographic_mode(false);
        entity.add_component(camera3d);

        scene.add_entity(&mut entity);
    }

    let mut random = Prng::new();
    for _ in 0..100 {
        let mut entity = Entity::new();

        let mut transform = Transform3D::new();
        transform.set_position(&[
            -5f32 + random.next_f32() * 10f32,
            -5f32 + random.next_f32() * 10f32,
            -5f32 + random.next_f32() * 10f32,
        ]);
        entity.add_component(transform);

        let mut material = Material::new();
        material.set_shader(Shader::new(VS_SRC, FS_SRC));

        entity.add_component(Sprite::new(material));

        scene.add_entity(&mut entity);
    }

    scene.init();

    unsafe {
        match window.make_current() {
            Ok(_) => {
                gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            },
            Err(e) => panic!("{:?}", e),
        }
    }

    scene_renderer.init();

    {
        let sprite_gl_renderer = scene_renderer.get_plugin::<GLRendererPlugin>().unwrap();
        let context = sprite_gl_renderer.get_context();
        println!(
            "OpenGL version: {:?}.{:?}, GLSL version {:?}.{:?}0",
            context.get_major(), context.get_minor(), context.get_glsl_major(), context.get_glsl_minor()
        );
    }

    let mut playing = true;
    while playing {
        let mut camera = scene.get_component_manager::<Camera3DManager>()
            .unwrap().get_active_camera().unwrap();

        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => {
                    playing = false;
                },
                glutin::Event::Resized(w, h) => {
                    camera.set(w as usize, h as usize);
                    scene_renderer.get_plugin::<GLRendererPlugin>()
                        .unwrap().get_context_mut().set_viewport(0, 0, w as usize, h as usize);
                },
                _ => (),
            }
        }

        let ms = now();
        let mut transform = camera.get_entity().unwrap().get_component::<Transform3D>().unwrap();

        transform.set_position(&[(ms.sin() * 10f64) as f32, (ms.cos() * 10f64) as f32, (ms.cos() * 10f64) as f32]);
        transform.look_at(&[0f32, 0f32, 0f32], &[0f32, 0f32, 1f32]);

        scene.update();
        scene_renderer.render();

        match window.swap_buffers() {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
    }

    scene.clear();
    scene_renderer.clear();
}

fn now() -> f64 {
    let current_time = time::get_time();
    (current_time.sec as f64) + (current_time.nsec as f64 / 1000f64 / 1000f64 / 1000f64)
}

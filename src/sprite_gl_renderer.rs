use core::mem;

use scene_graph::{Id, Component};
use scene_renderer::{SceneRenderer, Renderer};

use gl;
use gl::types::*;
use gl_renderer_plugin::GLRendererPlugin;

use geometry::{Attribute, Geometry};
use camera_components::{Camera3DManager, Camera2DManager};
use sprite_component::{Sprite, SpriteManager};
use transform_components::{Transform3D, Transform2D};

use mat3;
use mat4;

use shared::Shared;


struct SpriteGLRendererData {
    scene_renderer: Option<SceneRenderer>,
    geometry: Geometry,
}

#[derive(Clone)]
pub struct SpriteGLRenderer {
    data: Shared<SpriteGLRendererData>,
}

impl SpriteGLRenderer {
    pub fn new() -> Self {
        let mut geometry = Geometry::new();

        geometry
            .add_attribute(Attribute::new_f32("position", vector![
                -1f32, -1f32, 0f32,
                -1f32, 1f32, 0f32,
                1f32, 1f32, 0f32,
                1f32, -1f32, 0f32
            ], 3, false))
            .add_attribute(Attribute::new_f32("normal", vector![
                0f32, 0f32, 1f32,
                0f32, 0f32, 1f32,
                0f32, 0f32, 1f32,
                0f32, 0f32, 1f32
            ], 3, false))
            .add_attribute(Attribute::new_f32("tangent", vector![
                0f32, 0f32, 0f32, 1f32,
                0f32, 1f32, 0f32, 1f32,
                1f32, 1f32, 0f32, 1f32,
                1f32, 0f32, 0f32, 1f32
            ], 4, false))
            .add_attribute(Attribute::new_f32("uv", vector![
                0f32, 0f32,
                0f32, 1f32,
                1f32, 1f32,
                1f32, 0f32
            ], 2, false))
            .add_attribute(Attribute::new_f32("uv2", vector![
                0f32, 0f32,
                0f32, 1f32,
                1f32, 1f32,
                1f32, 0f32
            ], 2, false))
            .set_index(Attribute::new_u32("index", vector![
                0u32, 2u32, 1u32, 0u32, 3u32, 2u32
            ], 1, false));

        SpriteGLRenderer {
            data: Shared::new(SpriteGLRendererData {
                scene_renderer: None,
                geometry: geometry,
            })
        }
    }

    fn render_sprite(
        &mut self, sprite: &Sprite, view: &[f32; 16], projection: &[f32; 16], gl_plugin: &mut GLRendererPlugin
    ) -> &mut Self {
        if let Some(entity) = sprite.get_entity() {
            let mut model_view = mat4::new_identity();
            let mut normal = mat3::new_identity();

            if let Some(mut transform3d) = entity.get_component::<Transform3D>() {
                transform3d.get_model_view_matrix(&mut model_view, &view);
                transform3d.get_normal_matrix(&mut normal, &model_view);
            } else if let Some(mut transform2d) = entity.get_component::<Transform2D>() {
                transform2d.get_model_view_matrix(&mut model_view, &view);
                transform2d.get_normal_matrix(&mut normal, &model_view);
            }

            let mut gl_geometry = gl_plugin.get_geometry(&self.data.geometry);

            let material = sprite.get_material();
            let mut gl_material = gl_plugin.get_material(material);

            gl_plugin.get_context_mut().set_program(&gl_material.get_program(), false);

            gl_plugin.bind_material(&gl_material);
            gl_plugin.bind_uniforms(
                gl_material.get_program_mut(),
                projection,
                &model_view,
                view,
                &normal,
                false
            );
            gl_plugin.bind_attributes(&mut gl_geometry, gl_material.get_program_mut(), false);

            if gl_material.get_material().get_wireframe() {
                let context = gl_plugin.get_context_mut();
                let index_buffer = gl_geometry.get_line_index_buffer(context, false);
                context.set_buffer(index_buffer, false);
                unsafe {
                    gl::DrawElements(
                        gl::LINES,
                        index_buffer.get_length() as GLint,
                        gl::UNSIGNED_INT,
                        mem::transmute(0usize)
                    );
                }
            } else {
                let context = gl_plugin.get_context_mut();
                let index_buffer = gl_geometry.get_index_buffer(context, false);
                context.set_buffer(index_buffer, false);
                unsafe {
                    gl::DrawElements(
                        gl::TRIANGLES,
                        index_buffer.get_length() as GLint,
                        gl::UNSIGNED_INT,
                        mem::transmute(0usize)
                    );
                }
            }
        }
        self
    }
}

impl Renderer for SpriteGLRenderer {

    fn get_id(&self) -> Id { Id::of::<SpriteGLRenderer>() }

    fn get_scene_renderer(&self) -> Option<SceneRenderer> {
        self.data.scene_renderer.clone()
    }
    fn set_scene_renderer(&mut self, scene_renderer: Option<SceneRenderer>) {
        self.data.scene_renderer = scene_renderer;
    }

    fn get_order(&self) -> usize {0}

    fn clear(&mut self) {}
    fn init(&mut self) {}

    fn before_render(&mut self) {}
    fn after_render(&mut self) {}

    fn render(&mut self) {
        if let Some(scene_renderer) = self.get_scene_renderer() {
            let scene = scene_renderer.get_scene();

            let mut gl_plugin = scene_renderer.get_plugin::<GLRendererPlugin>().unwrap();
            let sprite_manager = scene.get_component_manager::<SpriteManager>().unwrap();

            let mut view = mat4::new_identity::<f32>();
            let mut projection = mat4::new_identity::<f32>();

            if let Some(camera3d_manager) = scene.get_component_manager::<Camera3DManager>() {
                if let Some(mut camera3d) = camera3d_manager.get_active_camera() {
                    mat4::copy(&mut view, camera3d.get_view());
                    mat4::copy(&mut projection, camera3d.get_projection());
                }
            } else if let Some(camera2d_manager) = scene.get_component_manager::<Camera2DManager>() {
                if let Some(mut camera2d) = camera2d_manager.get_active_camera() {
                    mat4::from_mat32(&mut view, camera2d.get_view());
                    mat4::from_mat32(&mut projection, camera2d.get_projection());
                }
            }

            for sprite in sprite_manager.get_components().iter() {
                self.render_sprite(sprite, &view, &projection, &mut gl_plugin);
            }
        }
    }
}

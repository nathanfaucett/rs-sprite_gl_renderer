#![no_std]


extern crate scene_graph;
extern crate scene_renderer;

extern crate gl;
extern crate gl_renderer_plugin;

extern crate geometry;

extern crate camera_components;
extern crate sprite_component;
extern crate transform_components;

#[macro_use]
extern crate vector;

extern crate mat3;
extern crate mat4;

extern crate shared;


mod sprite_gl_renderer;


pub use sprite_gl_renderer::SpriteGLRenderer;

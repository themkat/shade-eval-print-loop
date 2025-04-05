//! Command structures to standardize and simplify sending commands and instructions between threads.

use image::RgbaImage;
use nalgebra::Matrix4;

#[derive(Debug, PartialEq)]
pub enum UniformValue {
    Float(f32),
    Matrix(Matrix4<f32>),
    // TODO: other types
    // Texture1D?
    // Texture2D?
    Texture2D(RgbaImage),
    // etc.?
}

/// Commands meant to be received by the renderer.
#[derive(Debug, PartialEq)]
pub enum RenderCommand {
    // TODO: what kind of value?
    SetUniform(String, UniformValue),
} 

// TODO: decide on a structure for sending commands the other way
pub enum Command {}

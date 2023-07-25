use enum_map::Enum;

use ruffle_render::{commands::RenderBlendMode, pixel_bender::PixelBenderShaderHandle};
use swf::BlendMode;

#[derive(Enum, Debug, Copy, Clone)]
pub enum ComplexBlend {
    Multiply,   // Can't be trivial, 0 alpha is special case
    Lighten,    // Might be trivial but I can't reproduce the right colors
    Darken,     // Might be trivial but I can't reproduce the right colors
    Difference, // Can't be trivial, relies on abs operation
    Invert,     // May be trivial using a constant? Hard because it's without premultiplied alpha
    Alpha,      // Can't be trivial, requires layer tracking
    Erase,      // Can't be trivial, requires layer tracking
    Overlay,    // Can't be trivial, big math expression
    HardLight,  // Can't be trivial, big math expression
}

#[derive(Debug, Clone)]
pub enum BlendType {
    /// Trivial blends can be expressed with just a "draw bitmap" with blend states
    Trivial(TrivialBlend),

    /// Complex blends require a shader to express, so they are separated out into their own render
    Complex(ComplexBlend),

    /// Invoke a custom `PixelBender` shader.
    Shader(PixelBenderShaderHandle),
}

impl BlendType {
    pub fn from(mode: RenderBlendMode) -> BlendType {
        match mode {
            RenderBlendMode::Builtin(BlendMode::Normal) => BlendType::Trivial(TrivialBlend::Normal),
            RenderBlendMode::Builtin(BlendMode::Layer) => BlendType::Trivial(TrivialBlend::Normal),
            RenderBlendMode::Builtin(BlendMode::Multiply) => {
                BlendType::Complex(ComplexBlend::Multiply)
            }
            RenderBlendMode::Builtin(BlendMode::Screen) => BlendType::Trivial(TrivialBlend::Screen),
            RenderBlendMode::Builtin(BlendMode::Lighten) => {
                BlendType::Complex(ComplexBlend::Lighten)
            }
            RenderBlendMode::Builtin(BlendMode::Darken) => BlendType::Complex(ComplexBlend::Darken),
            RenderBlendMode::Builtin(BlendMode::Difference) => {
                BlendType::Complex(ComplexBlend::Difference)
            }
            RenderBlendMode::Builtin(BlendMode::Add) => BlendType::Trivial(TrivialBlend::Add),
            RenderBlendMode::Builtin(BlendMode::Subtract) => {
                BlendType::Trivial(TrivialBlend::Subtract)
            }
            RenderBlendMode::Builtin(BlendMode::Invert) => BlendType::Complex(ComplexBlend::Invert),
            RenderBlendMode::Builtin(BlendMode::Alpha) => BlendType::Complex(ComplexBlend::Alpha),
            RenderBlendMode::Builtin(BlendMode::Erase) => BlendType::Complex(ComplexBlend::Erase),
            RenderBlendMode::Builtin(BlendMode::Overlay) => {
                BlendType::Complex(ComplexBlend::Overlay)
            }
            RenderBlendMode::Builtin(BlendMode::HardLight) => {
                BlendType::Complex(ComplexBlend::HardLight)
            }
            RenderBlendMode::Shader(shader) => BlendType::Shader(shader),
        }
    }

    pub fn default_color(&self) -> wgpu::Color {
        wgpu::Color::TRANSPARENT
    }
}

#[derive(Enum, Debug, Copy, Clone)]
pub enum TrivialBlend {
    Normal,
    Add,
    Subtract,
    Screen,
}

impl TrivialBlend {
    pub fn blend_state(self) -> wgpu::BlendState {
        // out = <src_factor> * src <operation> <dst_factor> * dst
        match self {
            TrivialBlend::Normal => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            TrivialBlend::Add => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            TrivialBlend::Screen => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrc,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            TrivialBlend::Subtract => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
        }
    }
}

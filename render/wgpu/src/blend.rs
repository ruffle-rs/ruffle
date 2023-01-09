use enum_map::Enum;

use swf::BlendMode;

#[derive(Enum, Debug, Copy, Clone)]
pub enum ComplexBlend {
    Lighten,    // Might be trivial but I can't reproduce the right colors
    Darken,     // Might be trivial but I can't reproduce the right colors
    Difference, // Can't be trivial, relies on abs operation
    Invert,     // May be trivial using a constant? Hard because it's without premultiplied alpha
    Alpha,      // Can't be trivial, requires layer tracking
    Erase,      // Can't be trivial, requires layer tracking
    Overlay,    // Can't be trivial, big math expression
    HardLight,  // Can't be trivial, big math expression
}

#[derive(Debug, Copy, Clone)]
pub enum BlendType {
    /// Trivial blends can be expressed with just a "draw bitmap" with blend states
    Trivial(TrivialBlend),

    /// Complex blends require a shader to express, so they are separated out into their own render
    Complex(ComplexBlend),
}

impl BlendType {
    pub fn from(mode: BlendMode) -> BlendType {
        match mode {
            BlendMode::Normal => BlendType::Trivial(TrivialBlend::Normal),
            BlendMode::Layer => BlendType::Trivial(TrivialBlend::Normal),
            BlendMode::Multiply => BlendType::Trivial(TrivialBlend::Multiply),
            BlendMode::Screen => BlendType::Trivial(TrivialBlend::Screen),
            BlendMode::Lighten => BlendType::Complex(ComplexBlend::Lighten),
            BlendMode::Darken => BlendType::Complex(ComplexBlend::Darken),
            BlendMode::Difference => BlendType::Complex(ComplexBlend::Difference),
            BlendMode::Add => BlendType::Trivial(TrivialBlend::Add),
            BlendMode::Subtract => BlendType::Trivial(TrivialBlend::Subtract),
            BlendMode::Invert => BlendType::Complex(ComplexBlend::Invert),
            BlendMode::Alpha => BlendType::Complex(ComplexBlend::Alpha),
            BlendMode::Erase => BlendType::Complex(ComplexBlend::Erase),
            BlendMode::Overlay => BlendType::Complex(ComplexBlend::Overlay),
            BlendMode::HardLight => BlendType::Complex(ComplexBlend::HardLight),
        }
    }

    pub fn default_color(&self) -> wgpu::Color {
        match self {
            BlendType::Trivial(TrivialBlend::Multiply) => wgpu::Color::WHITE,
            _ => wgpu::Color::TRANSPARENT,
        }
    }
}

#[derive(Enum, Debug, Copy, Clone)]
pub enum TrivialBlend {
    Normal,
    Add,
    Subtract,
    Screen,
    Multiply,
}

impl TrivialBlend {
    pub fn blend_state(self) -> wgpu::BlendState {
        // out = <src_factor> * src <operation> <dst_factor> * dst
        match self {
            TrivialBlend::Normal => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            TrivialBlend::Multiply => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
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

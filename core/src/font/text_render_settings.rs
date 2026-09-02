use swf::TextGridFit;

/// The text rendering engine that a text field should use.
/// This is controlled by the "Anti-alias" setting in the Flash IDE.
/// Using "Anti-alias for readability" switches to the "Advanced" text
/// rendering engine.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextRenderSettings {
    /// This text should render with the standard rendering engine.
    /// Set via "Anti-alias for animation" in the Flash IDE.
    ///
    /// The `grid_fit`, `thickness`, and `sharpness` parameters are present
    /// because they are retained when switching from `Advanced` to `Normal`
    /// rendering and vice versa. They are not used in Normal rendering.
    Normal {
        grid_fit: TextGridFit,
        thickness: f32,
        sharpness: f32,
    },

    /// This text should render with the advanced rendering engine.
    /// Set via "Anti-alias for readability" in the Flash IDE.
    /// The parameters are set via the CSMTextSettings SWF tag.
    /// Ruffle does not support this currently, but this also affects
    /// hit-testing behavior.
    Advanced {
        grid_fit: TextGridFit,
        thickness: f32,
        sharpness: f32,
    },
}

impl TextRenderSettings {
    pub fn is_advanced(&self) -> bool {
        matches!(self, TextRenderSettings::Advanced { .. })
    }

    pub fn with_advanced_rendering(self) -> Self {
        match self {
            TextRenderSettings::Advanced { .. } => self,
            TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn with_normal_rendering(self) -> Self {
        match self {
            TextRenderSettings::Normal { .. } => self,
            TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn sharpness(&self) -> f32 {
        match self {
            TextRenderSettings::Normal { sharpness, .. } => *sharpness,
            TextRenderSettings::Advanced { sharpness, .. } => *sharpness,
        }
    }

    pub fn with_sharpness(self, sharpness: f32) -> Self {
        match self {
            TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness: _,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
            TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness: _,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn thickness(&self) -> f32 {
        match self {
            TextRenderSettings::Normal { thickness, .. } => *thickness,
            TextRenderSettings::Advanced { thickness, .. } => *thickness,
        }
    }

    pub fn with_thickness(self, thickness: f32) -> Self {
        match self {
            TextRenderSettings::Normal {
                grid_fit,
                thickness: _,
                sharpness,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
            TextRenderSettings::Advanced {
                grid_fit,
                thickness: _,
                sharpness,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn grid_fit(&self) -> swf::TextGridFit {
        match self {
            TextRenderSettings::Normal { grid_fit, .. } => *grid_fit,
            TextRenderSettings::Advanced { grid_fit, .. } => *grid_fit,
        }
    }

    pub fn with_grid_fit(self, grid_fit: TextGridFit) -> Self {
        match self {
            TextRenderSettings::Normal {
                grid_fit: _,
                thickness,
                sharpness,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
            TextRenderSettings::Advanced {
                grid_fit: _,
                thickness,
                sharpness,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }
}

impl From<swf::CsmTextSettings> for TextRenderSettings {
    fn from(settings: swf::CsmTextSettings) -> Self {
        if settings.use_advanced_rendering {
            TextRenderSettings::Advanced {
                grid_fit: settings.grid_fit,
                thickness: settings.thickness,
                sharpness: settings.sharpness,
            }
        } else {
            TextRenderSettings::default()
        }
    }
}

impl Default for TextRenderSettings {
    fn default() -> Self {
        Self::Normal {
            grid_fit: TextGridFit::Pixel,
            thickness: 0.0,
            sharpness: 0.0,
        }
    }
}

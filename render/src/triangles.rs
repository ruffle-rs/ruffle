use std::borrow::Cow;

use swf::{Point, Twips};

/// Triangle definitions for rendering triangles.
///
/// # Invariants
///
/// * `vertices`, `indices`, and `triangles` are non-empty.
/// * Every value in `indices` is `< vertices.len()`.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Triangles {
    /// Triangles described by shared `vertices` and triples of indices into
    /// that array.
    Indexed {
        vertices: Box<[Point<Twips>]>,
        indices: Box<[[u32; 3]]>,
    },
    /// Triangles described as independent vertex triples.
    Sequential { triangles: Box<[[Point<Twips>; 3]]> },
}

impl Triangles {
    pub fn points(&self) -> Box<dyn Iterator<Item = &Point<Twips>> + '_> {
        match self {
            Triangles::Indexed { vertices, .. } => Box::new(vertices.iter()),
            Triangles::Sequential { triangles } => {
                Box::new(triangles.iter().flat_map(|t| t.iter()))
            }
        }
    }

    pub fn vertices(&self) -> &[Point<Twips>] {
        match self {
            Triangles::Indexed { vertices, .. } => vertices,
            Triangles::Sequential { triangles } => triangles.as_flattened(),
        }
    }

    pub fn indices(&self) -> Cow<'_, [[u32; 3]]> {
        match self {
            Triangles::Indexed { indices, .. } => Cow::Borrowed(indices),
            Triangles::Sequential { triangles } => Cow::Owned(
                (0..triangles.len() as u32)
                    .map(|i| {
                        let b = i * 3;
                        [b, b + 1, b + 2]
                    })
                    .collect(),
            ),
        }
    }
}

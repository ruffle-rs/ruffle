use crate::descriptors::Descriptors;
use crate::globals::Globals;
use fnv::FnvHashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

type PoolInner<T> = Mutex<Vec<T>>;
type Constructor<T> = Box<dyn Fn(&Descriptors) -> T>;

#[derive(Debug)]
pub struct TexturePool {
    pools: FnvHashMap<TextureKey, BufferPool<wgpu::Texture>>,
    globals_cache: FnvHashMap<GlobalsKey, Weak<Globals>>,
}

impl TexturePool {
    pub fn new() -> Self {
        Self {
            pools: FnvHashMap::default(),
            globals_cache: FnvHashMap::default(),
        }
    }

    pub fn get_texture(
        &mut self,
        descriptors: &Descriptors,
        size: wgpu::Extent3d,
        usage: wgpu::TextureUsages,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> PoolEntry<wgpu::Texture> {
        let key = TextureKey {
            size,
            usage,
            format,
            sample_count,
        };
        let pool = self.pools.entry(key).or_insert_with(|| {
            let label = if cfg!(feature = "render_debug_labels") {
                use std::sync::atomic::{AtomicU32, Ordering};
                static ID_COUNT: AtomicU32 = AtomicU32::new(0);
                let id = ID_COUNT.fetch_add(1, Ordering::Relaxed);
                create_debug_label!("Pooled texture {}", id)
            } else {
                None
            };
            BufferPool::new(Box::new(move |descriptors| {
                descriptors.device.create_texture(&wgpu::TextureDescriptor {
                    label: label.as_deref(),
                    size,
                    mip_level_count: 1,
                    sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                })
            }))
        });
        pool.take(&descriptors)
    }

    pub fn get_globals(
        &mut self,
        descriptors: &Descriptors,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Arc<Globals> {
        let globals = self
            .globals_cache
            .entry(GlobalsKey {
                viewport_width,
                viewport_height,
            })
            .or_default();
        match globals.upgrade() {
            None => {
                let replacement = Arc::new(Globals::new(
                    &descriptors.device,
                    &descriptors.bind_layouts.globals,
                    viewport_width,
                    viewport_height,
                ));
                *globals = Arc::downgrade(&replacement);
                replacement
            }
            Some(globals) => globals,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct TextureKey {
    size: wgpu::Extent3d,
    usage: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
    sample_count: u32,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct GlobalsKey {
    viewport_width: u32,
    viewport_height: u32,
}

pub struct BufferPool<T> {
    available: Arc<PoolInner<T>>,
    constructor: Constructor<T>,
}

impl<T> Debug for BufferPool<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferPool").finish()
    }
}

impl<T> BufferPool<T> {
    pub fn new(constructor: Constructor<T>) -> Self {
        Self {
            available: Arc::new(Mutex::new(vec![])),
            constructor,
        }
    }

    pub fn take(&self, descriptors: &Descriptors) -> PoolEntry<T> {
        let item = self
            .available
            .lock()
            .expect("Should not be able to lock recursively")
            .pop()
            .unwrap_or_else(|| (self.constructor)(descriptors));
        PoolEntry {
            item: Some(item),
            pool: Arc::downgrade(&self.available),
        }
    }
}

pub struct PoolEntry<T> {
    item: Option<T>,
    pool: Weak<PoolInner<T>>,
}

impl<T> Debug for PoolEntry<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PoolEntry").field(&self.item).finish()
    }
}

impl<T> Drop for PoolEntry<T> {
    fn drop(&mut self) {
        if let Some(item) = self.item.take() {
            if let Some(pool) = self.pool.upgrade() {
                pool.lock()
                    .expect("Should not be able to lock recursively")
                    .push(item)
            }
        }
    }
}

impl<T> Deref for PoolEntry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().expect("Item should exist until dropped")
    }
}

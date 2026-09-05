use super::BitmapFormat;
use crate::backend::RenderBackend;
use crate::bitmap::{Bitmap, BitmapHandle, PixelRegion};
use guillotiere::{Allocation, AtlasAllocator};
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

struct BitmapAtlasData {
    bitmap: Bitmap<'static>,
    allocator: AtlasAllocator,
    handle: OnceCell<Option<BitmapHandle>>,

    /// Bounding box of CPU-side changes made since `handle` was last synced to
    /// the renderer. `None` means there's nothing pending.
    dirty: Option<PixelRegion>,
}

/// Bitmap atlas is a bitmap with allocable regions. Instead of having many
/// small bitmaps, you can use a bitmap atlas with one large bitmap.
///
/// The atlas allows dynamic allocation and deallocation.
#[derive(Clone)]
pub struct BitmapAtlas(Rc<RefCell<BitmapAtlasData>>);

impl BitmapAtlas {
    #[expect(clippy::unwrap_used)]
    pub fn new(width: u32, height: u32, format: BitmapFormat) -> Self {
        Self(Rc::new(RefCell::new(BitmapAtlasData {
            bitmap: Bitmap::new(
                width,
                height,
                format,
                vec![
                    0u8;
                    format.length_for_size(width.try_into().unwrap(), height.try_into().unwrap())
                ],
            ),
            allocator: AtlasAllocator::new(to_size2(width, height)),
            handle: OnceCell::new(),
            dirty: None,
        })))
    }

    /// Allocate a new region in this atlas.
    ///
    /// Note that the returned region may contain garbage from previous
    /// allocations. Set `clear` to `true` to clear the region before returning.
    ///
    /// Returns [`None`] when there's no space left to allocate.
    pub fn allocate(&self, width: u32, height: u32, clear: bool) -> Option<BitmapAtlasRegion> {
        let allocation = self
            .0
            .borrow_mut()
            .allocator
            .allocate(to_size2(width, height))?;
        let region = BitmapAtlasRegion {
            atlas: self.clone(),
            allocation,
            leaked: false,
        };
        if clear {
            region.clear();
        }
        Some(region)
    }

    /// Returns the handle for this atlas's texture, registering it on first
    /// use and pushing any pending CPU-side changes since the last call.
    pub fn handle(&self, renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        let mut data = self.0.borrow_mut();

        let handle = if let Some(handle) = data.handle.get() {
            handle
        } else {
            // First use: registering captures the bitmap's current state,
            // so there's nothing left to separately push as an update.
            data.dirty = None;

            let bitmap = data.bitmap.reborrow();
            data.handle.get_or_init(|| {
                renderer
                    .register_bitmap(bitmap)
                    .inspect_err(|e| tracing::error!("Failed to register a bitmap atlas: {e}"))
                    .ok()
            })
        }
        .clone();

        if let Some(dirty) = data.dirty.take()
            && let Some(handle) = handle.as_ref()
        {
            let bitmap = data.bitmap.reborrow();
            if let Err(e) = renderer.update_texture(handle, bitmap, dirty) {
                tracing::error!("Failed to update atlas texture: {e}");
            }
        }

        handle
    }

    pub fn format(&self) -> BitmapFormat {
        self.0.borrow().bitmap.format()
    }

    /// This page's pixels, converted to RGBA. Intended for debug/inspection
    /// purposes; rendering goes through `handle()` instead.
    pub fn to_rgba(&self) -> Bitmap<'static> {
        self.0.borrow().bitmap.clone().to_rgba()
    }
}

impl std::fmt::Debug for BitmapAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.0.borrow();
        f.debug_struct("BitmapAtlas")
            .field("ptr", &Rc::as_ptr(&self.0))
            .field("width", &data.bitmap.width())
            .field("height", &data.bitmap.height())
            .field("format", &data.bitmap.format())
            .finish()
    }
}

/// A region allocated within a [`BitmapAtlas`].
///
/// Owns its allocation: the space is freed back to the atlas's allocator
/// when this value is dropped, unless [`BitmapAtlasRegion::leak`] is called
/// on it instead.
pub struct BitmapAtlasRegion {
    atlas: BitmapAtlas,
    allocation: Allocation,
    leaked: bool,
}

impl BitmapAtlasRegion {
    pub fn atlas(&self) -> &BitmapAtlas {
        &self.atlas
    }

    pub fn region(&self) -> PixelRegion {
        to_pixel_region(self.allocation.rectangle)
    }

    /// Prevents this region's space from being freed back to the atlas's
    /// allocator when this value is dropped.
    pub fn leak(mut self) {
        self.leaked = true;
    }

    /// Clear the whole region.
    pub fn clear(&self) {
        let region = self.region();

        let mut atlas = self.atlas.0.borrow_mut();
        atlas
            .bitmap
            .clear_region(region.x_min, region.y_min, region.width(), region.height());
        match &mut atlas.dirty {
            Some(dirty) => dirty.union(region),
            None => atlas.dirty = Some(region),
        }
    }

    /// Set the region with the given bitmap.
    pub fn set_region(&self, source: &Bitmap<'_>, x: u32, y: u32) {
        let region = self.region();
        let (x, y) = (region.x_min + x, region.y_min + y);
        assert!(x + source.width <= region.x_max);
        assert!(y + source.height <= region.y_max);

        let mut atlas = self.atlas.0.borrow_mut();
        atlas.bitmap.set_region(source, x, y);
        match &mut atlas.dirty {
            Some(dirty) => dirty.union(region),
            None => atlas.dirty = Some(region),
        }
    }
}

impl Drop for BitmapAtlasRegion {
    fn drop(&mut self) {
        if self.leaked {
            return;
        }
        self.atlas
            .0
            .borrow_mut()
            .allocator
            .deallocate(self.allocation.id);
    }
}

fn to_size2(width: u32, height: u32) -> guillotiere::Size {
    guillotiere::size2(
        width.try_into().expect("bitmap size"),
        height.try_into().expect("bitmap size"),
    )
}

fn to_pixel_region(rect: guillotiere::Rectangle) -> PixelRegion {
    PixelRegion::for_region_i32(
        rect.x_range().start,
        rect.y_range().start,
        rect.width(),
        rect.height(),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::{
        BitmapCacheEntry, Context3D, Context3DProfile, PixelBenderOutput, PixelBenderTarget,
        ShapeHandle, ViewportDimensions,
    };
    use crate::bitmap::{BitmapHandleImpl, BitmapSource, RgbaBufRead, SyncHandle};
    use crate::commands::CommandList;
    use crate::error::Error;
    use crate::pixel_bender::{PixelBenderShader, PixelBenderShaderHandle};
    use crate::pixel_bender_support::PixelBenderShaderArgument;
    use crate::quality::StageQuality;
    use crate::shape_utils::DistilledShape;
    use std::borrow::Cow;
    use std::num::NonZeroU32;
    use std::sync::Arc;
    use swf::Color;

    fn solid_bitmap(width: u32, height: u32, format: BitmapFormat, fill: u8) -> Bitmap<'static> {
        let len = format.length_for_size(width as usize, height as usize);
        Bitmap::new(width, height, format, vec![fill; len])
    }

    #[derive(Debug)]
    struct TestBitmapHandle;
    impl BitmapHandleImpl for TestBitmapHandle {}

    /// A minimal `RenderBackend` that records `register_bitmap`/`update_texture`
    /// calls so `BitmapAtlas::handle`'s caching and dirty-tracking behavior can
    /// be observed. Every other method is unreachable from these tests.
    #[derive(Default)]
    struct RecordingBackend {
        fail_register: bool,

        register_calls: RefCell<u32>,
        last_registered: RefCell<Option<Vec<u8>>>,

        update_calls: RefCell<Vec<PixelRegion>>,
        last_updated: RefCell<Option<Vec<u8>>>,
    }

    impl RenderBackend for RecordingBackend {
        fn viewport_dimensions(&self) -> ViewportDimensions {
            unimplemented!()
        }
        fn set_viewport_dimensions(&mut self, _dimensions: ViewportDimensions) {
            unimplemented!()
        }
        fn register_shape(
            &mut self,
            _shape: DistilledShape,
            _bitmap_source: &dyn BitmapSource,
        ) -> ShapeHandle {
            unimplemented!()
        }
        fn render_offscreen(
            &mut self,
            _handle: BitmapHandle,
            _commands: CommandList,
            _quality: StageQuality,
            _bounds: PixelRegion,
            _cache_entries: Vec<BitmapCacheEntry>,
        ) -> Option<Box<dyn SyncHandle>> {
            unimplemented!()
        }
        fn submit_frame(
            &mut self,
            _clear: Color,
            _commands: CommandList,
            _cache_entries: Vec<BitmapCacheEntry>,
        ) {
            unimplemented!()
        }
        fn create_empty_texture(
            &mut self,
            _width: NonZeroU32,
            _height: NonZeroU32,
        ) -> Result<BitmapHandle, Error> {
            unimplemented!()
        }
        fn register_bitmap(&mut self, bitmap: Bitmap<'_>) -> Result<BitmapHandle, Error> {
            *self.register_calls.borrow_mut() += 1;
            if self.fail_register {
                return Err(Error::Unimplemented("test failure".into()));
            }
            *self.last_registered.borrow_mut() = Some(bitmap.data().to_vec());
            Ok(BitmapHandle(Arc::new(TestBitmapHandle)))
        }
        fn update_texture(
            &mut self,
            _handle: &BitmapHandle,
            bitmap: Bitmap<'_>,
            region: PixelRegion,
        ) -> Result<(), Error> {
            self.update_calls.borrow_mut().push(region);
            *self.last_updated.borrow_mut() = Some(bitmap.data().to_vec());
            Ok(())
        }
        fn create_context3d(
            &mut self,
            _profile: Context3DProfile,
        ) -> Result<Box<dyn Context3D>, Error> {
            unimplemented!()
        }
        fn debug_info(&self) -> Cow<'static, str> {
            unimplemented!()
        }
        fn name(&self) -> &'static str {
            "test"
        }
        fn set_quality(&mut self, _quality: StageQuality) {
            unimplemented!()
        }
        fn compile_pixelbender_shader(
            &mut self,
            _shader: PixelBenderShader,
        ) -> Result<PixelBenderShaderHandle, Error> {
            unimplemented!()
        }
        fn run_pixelbender_shader(
            &mut self,
            _shader: PixelBenderShaderHandle,
            _arguments: &[PixelBenderShaderArgument],
            _target: &PixelBenderTarget,
        ) -> Result<PixelBenderOutput, Error> {
            unimplemented!()
        }
        fn resolve_sync_handle(
            &mut self,
            _handle: Box<dyn SyncHandle>,
            _with_rgba: RgbaBufRead,
        ) -> Result<(), Error> {
            unimplemented!()
        }
    }

    #[test]
    fn new_creates_zeroed_bitmap_with_correct_format_and_size() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        assert_eq!(atlas.format(), BitmapFormat::Rgba);

        let bitmap = atlas.to_rgba();
        assert_eq!(bitmap.width(), 4);
        assert_eq!(bitmap.height(), 4);
        assert!(bitmap.data().iter().all(|&b| b == 0));
    }

    #[test]
    fn to_rgba_does_not_mutate_original_atlas_bitmap() {
        let atlas = BitmapAtlas::new(1, 1, BitmapFormat::Rgb);
        let _ = atlas.to_rgba();
        assert_eq!(atlas.format(), BitmapFormat::Rgb);
    }

    #[test]
    fn cloned_atlas_shares_underlying_state() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let clone = atlas.clone();

        let region = atlas
            .allocate(2, 2, false)
            .expect("allocation should succeed");
        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0x55);
        region.set_region(&source, 0, 0);

        assert!(clone.to_rgba().data().iter().all(|&b| b == 0x55));
    }

    #[test]
    fn allocate_returns_region_containing_requested_size() {
        let atlas = BitmapAtlas::new(16, 16, BitmapFormat::Rgba);
        let region = atlas
            .allocate(4, 4, false)
            .expect("allocation should succeed");

        let r = region.region();
        assert_eq!(r.width(), 4);
        assert_eq!(r.height(), 4);
        assert!(r.x_max <= 16);
        assert!(r.y_max <= 16);
    }

    #[test]
    fn allocate_whole_atlas_returns_exact_bounds() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let region = atlas
            .allocate(4, 4, false)
            .expect("allocation should succeed");
        assert_eq!(region.region(), PixelRegion::for_whole_size(4, 4));
    }

    #[test]
    fn allocate_returns_none_when_atlas_is_full() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let _region = atlas
            .allocate(4, 4, false)
            .expect("first allocation should succeed");
        assert!(atlas.allocate(1, 1, false).is_none());
    }

    #[test]
    fn allocate_returns_none_when_atlas_does_not_have_enough_space() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);

        let region1 = atlas.allocate(3, 3, false);
        assert!(region1.is_some());

        let region2 = atlas.allocate(2, 2, false);
        assert!(region2.is_none());
        let region3 = atlas.allocate(1, 1, false);
        assert!(region3.is_some());
        let region4 = atlas.allocate(1, 1, false);
        assert!(region4.is_some());
        let region5 = atlas.allocate(2, 1, false);
        assert!(region5.is_some());
        let region6 = atlas.allocate(2, 2, false);
        assert!(region6.is_none());
    }

    #[test]
    fn allocate_returns_non_overlapping_regions() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);

        let region1 = atlas.allocate(2, 2, false).expect("should succeed");
        let region2 = atlas.allocate(2, 2, false).expect("should succeed");
        let region3 = atlas.allocate(2, 2, false).expect("should succeed");
        let region4 = atlas.allocate(2, 2, false).expect("should succeed");

        region1.set_region(&solid_bitmap(2, 2, BitmapFormat::Rgba, 1), 0, 0);
        region2.set_region(&solid_bitmap(2, 2, BitmapFormat::Rgba, 2), 0, 0);
        region3.set_region(&solid_bitmap(2, 2, BitmapFormat::Rgba, 3), 0, 0);
        region4.set_region(&solid_bitmap(2, 2, BitmapFormat::Rgba, 4), 0, 0);

        assert!(atlas.to_rgba().data().iter().all(|&b| b != 0));
    }

    #[test]
    fn dropping_region_frees_space_for_reuse() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let region = atlas
            .allocate(4, 4, false)
            .expect("allocation should succeed");
        assert!(atlas.allocate(1, 1, false).is_none());

        drop(region);

        assert!(atlas.allocate(4, 4, false).is_some());
    }

    #[test]
    fn leaked_region_does_not_free_space_for_reuse() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let region = atlas
            .allocate(4, 4, false)
            .expect("allocation should succeed");

        region.leak();

        assert!(atlas.allocate(1, 1, false).is_none());
    }

    #[test]
    fn region_shares_allocator_state_with_originating_atlas() {
        let atlas = BitmapAtlas::new(2, 1, BitmapFormat::Rgba);
        let region_a = atlas
            .allocate(1, 1, false)
            .expect("region a should allocate");
        let region_b = region_a
            .atlas()
            .allocate(1, 1, false)
            .expect("region b should allocate via region_a's atlas handle");

        assert_ne!(
            region_a.region(),
            region_b.region(),
            "the two regions must occupy distinct space in the shared allocator"
        );
        assert!(
            atlas.allocate(1, 1, false).is_none(),
            "atlas should now be full"
        );
    }

    #[test]
    fn set_region_writes_pixels_at_given_offset() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let region = atlas
            .allocate(4, 4, true)
            .expect("allocation should succeed");

        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 7);
        region.set_region(&source, 1, 1);

        let bitmap = atlas.to_rgba();
        #[rustfmt::skip]
        let expected: [u8; 64] = [
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 7,7,7,7, 7,7,7,7, 0,0,0,0,
            0,0,0,0, 7,7,7,7, 7,7,7,7, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
        ];
        assert_eq!(bitmap.data(), &expected);
    }

    #[test]
    #[should_panic]
    fn set_region_panics_when_source_exceeds_region_bounds() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let region = atlas
            .allocate(2, 2, false)
            .expect("allocation should succeed");
        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0x11);
        // Offset (1,1) + a 2x2 source overflows the 2x2 region.
        region.set_region(&source, 1, 1);
    }

    #[test]
    fn clear_zeroes_pixel_data_in_region() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let region = atlas
            .allocate(2, 2, false)
            .expect("allocation should succeed");
        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0xFF);
        region.set_region(&source, 0, 0);
        assert!(atlas.to_rgba().data().iter().all(|&b| b == 0xFF));

        region.clear();
        assert!(atlas.to_rgba().data().iter().all(|&b| b == 0));
    }

    #[test]
    fn allocate_with_clear_zeroes_reused_space() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let region = atlas
            .allocate(2, 2, false)
            .expect("allocation should succeed");
        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0x77);
        region.set_region(&source, 0, 0);
        drop(region);

        assert!(atlas.to_rgba().data().iter().all(|&b| b == 0x77));

        let region = atlas
            .allocate(2, 2, true)
            .expect("reallocation should succeed");
        assert_eq!(region.region(), PixelRegion::for_whole_size(2, 2));
        assert!(atlas.to_rgba().data().iter().all(|&b| b == 0));
    }

    #[test]
    fn allocate_without_clear_can_contain_leftover_data() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let region = atlas
            .allocate(2, 2, false)
            .expect("allocation should succeed");
        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0x77);
        region.set_region(&source, 0, 0);
        drop(region);

        let region = atlas
            .allocate(2, 2, false)
            .expect("reallocation should succeed");
        assert_eq!(region.region(), PixelRegion::for_whole_size(2, 2));
        assert!(
            atlas.to_rgba().data().iter().all(|&b| b == 0x77),
            "unclear reallocation should retain previous contents"
        );
    }

    #[test]
    fn handle_is_cached_across_calls() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let mut backend = RecordingBackend::default();

        let first = atlas.handle(&mut backend);
        let second = atlas.handle(&mut backend);

        assert_eq!(first, second);
        assert_eq!(*backend.register_calls.borrow(), 1);
        assert!(backend.update_calls.borrow().is_empty());
    }

    #[test]
    fn handle_registers_bitmap_on_first_use_without_separate_update() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let region = atlas
            .allocate(2, 2, true)
            .expect("allocation should succeed");
        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0x42);
        // Marks the atlas dirty before it's ever been registered.
        region.set_region(&source, 0, 0);

        let mut backend = RecordingBackend::default();

        assert!(atlas.handle(&mut backend).is_some());
        assert!(atlas.handle(&mut backend).is_some());

        assert_eq!(*backend.register_calls.borrow(), 1);
        assert!(
            backend.update_calls.borrow().is_empty(),
            "first registration already captures pending changes, so no separate update should be pushed"
        );
        assert_eq!(
            backend.last_registered.borrow().as_deref(),
            Some([0x42; 16].as_slice())
        );
    }

    #[test]
    fn handle_pushes_update_texture_for_changes_after_registration() {
        let atlas = BitmapAtlas::new(4, 2, BitmapFormat::Rgba);
        let _region_a = atlas
            .allocate(2, 2, true)
            .expect("region a should allocate");
        let region_b = atlas
            .allocate(2, 2, true)
            .expect("region b should allocate");

        let mut backend = RecordingBackend::default();

        // Initial registration; nothing dirty afterwards.
        atlas.handle(&mut backend);

        assert_eq!(*backend.register_calls.borrow(), 1, "should register");
        assert_eq!(backend.update_calls.borrow().len(), 0, "should not update");

        let source = solid_bitmap(2, 2, BitmapFormat::Rgba, 0x09);
        region_b.set_region(&source, 0, 0);

        atlas.handle(&mut backend);

        assert_eq!(
            *backend.register_calls.borrow(),
            1,
            "should not re-register on subsequent calls"
        );
        assert_eq!(
            backend.update_calls.borrow().as_slice(),
            &[region_b.region()],
            "dirty region pushed to the renderer should match the region that changed"
        );
        // `update_texture` is handed the whole atlas bitmap (the renderer uses
        // the accompanying region to know which part actually changed).
        assert_eq!(
            backend.last_updated.borrow().as_deref(),
            Some(atlas.to_rgba().data())
        );
    }

    #[test]
    fn handle_unions_multiple_dirty_regions_into_one_update() {
        let atlas = BitmapAtlas::new(4, 4, BitmapFormat::Rgba);
        let region_a = atlas
            .allocate(2, 2, true)
            .expect("region a should allocate");
        let region_b = atlas
            .allocate(2, 2, true)
            .expect("region b should allocate");
        let region_c = atlas
            .allocate(2, 2, true)
            .expect("region c should allocate");

        let mut backend = RecordingBackend::default();
        atlas.handle(&mut backend); // Initial registration.

        region_a.clear();
        region_b.clear();
        region_c.clear();
        atlas.handle(&mut backend);

        assert_eq!(
            backend.update_calls.borrow().as_slice(),
            &[PixelRegion::for_whole_size(4, 4)]
        );
    }

    #[test]
    fn handle_returns_none_and_does_not_retry_after_registration_failure() {
        let atlas = BitmapAtlas::new(2, 2, BitmapFormat::Rgba);
        let mut backend = RecordingBackend {
            fail_register: true,
            ..Default::default()
        };

        assert!(atlas.handle(&mut backend).is_none());
        assert!(atlas.handle(&mut backend).is_none());
        assert_eq!(
            *backend.register_calls.borrow(),
            1,
            "a failed registration should be cached, not retried"
        );
    }

    #[test]
    fn debug_format_includes_dimensions_and_format() {
        let atlas = BitmapAtlas::new(8, 6, BitmapFormat::Rgba);
        let ptr = Rc::as_ptr(&atlas.0);
        assert_eq!(
            format!("{atlas:?}"),
            format!("BitmapAtlas {{ ptr: {ptr:?}, width: 8, height: 6, format: Rgba }}")
        );
    }
}

mod avm1;
mod avm2;
mod display_object;
mod domain;
mod handle;
mod movie;

use crate::context::{RenderContext, UpdateContext};
use crate::debug_ui::avm1::Avm1ObjectWindow;
use crate::debug_ui::avm2::Avm2ObjectWindow;
use crate::debug_ui::display_object::{DisplayObjectSearchWindow, DisplayObjectWindow};
use crate::debug_ui::domain::DomainListWindow;
use crate::debug_ui::handle::{
    AVM1ObjectHandle, AVM2ObjectHandle, DisplayObjectHandle, DomainHandle,
};
use crate::debug_ui::movie::{MovieListWindow, MovieWindow};
use crate::display_object::TDisplayObject;
use crate::tag_utils::SwfMovie;
use gc_arena::DynamicRootSet;
use hashbrown::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Weak};
use swf::{Color, Rectangle, Twips};
use weak_table::PtrWeakKeyHashMap;

#[derive(Default)]
pub struct DebugUi {
    display_objects: HashMap<DisplayObjectHandle, DisplayObjectWindow>,
    movies: PtrWeakKeyHashMap<Weak<SwfMovie>, MovieWindow>,
    avm1_objects: HashMap<AVM1ObjectHandle, Avm1ObjectWindow>,
    avm2_objects: HashMap<AVM2ObjectHandle, Avm2ObjectWindow>,
    domains: HashMap<DomainHandle, DomainListWindow>,
    queued_messages: Vec<Message>,
    items_to_save: Vec<ItemToSave>,
    movie_list: Option<MovieListWindow>,
    domain_list: Option<DomainListWindow>,
    display_object_search: Option<DisplayObjectSearchWindow>,
}

#[derive(Debug)]
pub enum Message {
    TrackDisplayObject(DisplayObjectHandle),
    TrackDomain(DomainHandle),
    TrackMovie(Arc<SwfMovie>),
    TrackAVM1Object(AVM1ObjectHandle),
    TrackAVM2Object(AVM2ObjectHandle),
    TrackStage,
    TrackTopLevelMovie,
    ShowKnownMovies,
    ShowDomains,
    SaveFile(ItemToSave),
    SearchForDisplayObject,
}

impl DebugUi {
    pub(crate) fn show(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext,
        movie_offset: f64,
    ) {
        let mut messages = std::mem::take(&mut self.queued_messages);

        self.display_objects.retain(|object, window| {
            let object = object.fetch(context.dynamic_root);
            window.show(egui_ctx, context, object, &mut messages)
        });

        self.avm1_objects.retain(|object, window| {
            let object = object.fetch(context.dynamic_root);
            window.show(egui_ctx, context, object, &mut messages)
        });

        self.avm2_objects.retain(|object, window| {
            let object = object.fetch(context.dynamic_root);
            window.show(egui_ctx, context, object, &mut messages)
        });

        self.movies
            .retain(|movie, window| window.show(egui_ctx, context, movie, &mut messages));

        if let Some(mut movie_list) = self.movie_list.take() {
            if movie_list.show(egui_ctx, context, &mut messages) {
                self.movie_list = Some(movie_list);
            }
        }

        if let Some(mut domain_list) = self.domain_list.take() {
            if domain_list.show(egui_ctx, context, &mut messages) {
                self.domain_list = Some(domain_list);
            }
        }

        if let Some(mut search) = self.display_object_search.take() {
            if search.show(egui_ctx, context, &mut messages, movie_offset) {
                self.display_object_search = Some(search);
            }
        }

        for message in messages {
            match message {
                Message::TrackDisplayObject(object) => {
                    self.track_display_object(object);
                }
                Message::TrackDomain(domain) => {
                    self.domains.insert(domain, Default::default());
                }
                Message::TrackStage => {
                    self.track_display_object(DisplayObjectHandle::new(context, context.stage));
                }
                Message::TrackMovie(movie) => {
                    self.movies.insert(movie, Default::default());
                }
                Message::TrackTopLevelMovie => {
                    self.movies.insert(context.swf.clone(), Default::default());
                }
                Message::TrackAVM1Object(object) => {
                    self.avm1_objects.insert(object, Default::default());
                }
                Message::TrackAVM2Object(object) => {
                    self.avm2_objects.insert(object, Default::default());
                }
                Message::SaveFile(file) => {
                    self.items_to_save.push(file);
                }
                Message::ShowKnownMovies => {
                    self.movie_list = Some(Default::default());
                }
                Message::ShowDomains => {
                    self.domain_list = Some(Default::default());
                }
                Message::SearchForDisplayObject => {
                    self.display_object_search = Some(Default::default());
                }
            }
        }
    }

    pub fn should_suspend_player(&self) -> bool {
        self.display_object_search.is_some()
    }

    pub fn items_to_save(&mut self) -> Vec<ItemToSave> {
        std::mem::take(&mut self.items_to_save)
    }

    pub fn queue_message(&mut self, message: Message) {
        self.queued_messages.push(message);
    }

    pub fn track_display_object(&mut self, handle: DisplayObjectHandle) {
        self.display_objects.insert(handle, Default::default());
    }

    pub fn draw_debug_rects<'gc>(
        &self,
        context: &mut RenderContext<'_, 'gc>,
        dynamic_root_set: DynamicRootSet<'gc>,
    ) {
        let world_matrix = context.stage.view_matrix() * *context.stage.base().matrix();

        for (object, window) in self.display_objects.iter() {
            if let Some(color) = window.debug_rect_color() {
                let object = object.fetch(dynamic_root_set);
                let bounds = world_matrix * object.debug_rect_bounds();

                draw_debug_rect(context, color, bounds, 3.0);
            }

            if let Some(object) = window.hovered_debug_rect() {
                let object = object.fetch(dynamic_root_set);
                let bounds = world_matrix * object.debug_rect_bounds();

                draw_debug_rect(context, Color::RED, bounds, 5.0);
            }

            if let Some(bounds) = window.hovered_bounds() {
                let bounds = world_matrix * bounds;
                draw_debug_rect(context, Color::RED, bounds, 5.0);
            }
        }

        if let Some(window) = &self.display_object_search {
            for (color, object) in window.hovered_debug_rects() {
                let object = object.fetch(dynamic_root_set);
                let bounds = world_matrix * object.debug_rect_bounds();

                draw_debug_rect(context, color, bounds, 5.0);
            }
        }

        for (_object, window) in self.avm1_objects.iter() {
            if let Some(object) = window.hovered_debug_rect() {
                let object = object.fetch(dynamic_root_set);
                let bounds = world_matrix * object.debug_rect_bounds();

                draw_debug_rect(context, Color::RED, bounds, 5.0);
            }
        }

        for (_object, window) in self.avm2_objects.iter() {
            if let Some(object) = window.hovered_debug_rect() {
                let object = object.fetch(dynamic_root_set);
                let bounds = world_matrix * object.debug_rect_bounds();

                draw_debug_rect(context, Color::RED, bounds, 5.0);
            }
        }
    }
}

pub struct ItemToSave {
    pub suggested_name: String,
    pub data: Vec<u8>,
}

impl Debug for ItemToSave {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemToSave")
            .field("suggested_name", &self.suggested_name)
            .field("data", &self.data.len())
            .finish()
    }
}

fn draw_debug_rect(
    context: &mut RenderContext,
    color: Color,
    bounds: Rectangle<Twips>,
    thickness: f32,
) {
    let thickness = Twips::from_pixels(thickness as f64);
    let bounds = bounds.grow(thickness);
    context.draw_rect_outline(color, bounds, thickness);
}

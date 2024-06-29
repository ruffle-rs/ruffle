use crate::context::UpdateContext;
use crate::debug_ui::display_object::{open_display_object_button, DEFAULT_DEBUG_COLORS};
use crate::debug_ui::handle::DisplayObjectHandle;
use crate::debug_ui::Message;
use crate::display_object::{
    DisplayObject, TDisplayObject, TDisplayObjectContainer, TInteractiveObject,
};
use egui::collapsing_header::CollapsingState;
use egui::color_picker::show_color;
use egui::{Rgba, Ui, Vec2, Window};
use fnv::FnvHashMap;
use swf::{Point, Twips};

#[derive(Debug)]
struct DisplayObjectTree {
    handle: DisplayObjectHandle,
    children: Vec<DisplayObjectTree>,
    color: [f32; 3],
}

#[derive(Debug, Default)]
pub struct DisplayObjectSearchWindow {
    finding: bool,
    results: Vec<DisplayObjectTree>,
    unique_results: FnvHashMap<DisplayObjectHandle, swf::Color>,
    hovered_debug_rect: Option<DisplayObjectHandle>,
    include_hidden: bool,
    only_mouse_enabled: bool,
}

impl DisplayObjectSearchWindow {
    pub fn hovered_debug_rects(&self) -> Vec<(swf::Color, DisplayObjectHandle)> {
        if let Some(hovered_debug_rect) = &self.hovered_debug_rect {
            vec![(swf::Color::RED, hovered_debug_rect.clone())]
        } else {
            self.unique_results
                .iter()
                .map(|(k, v)| (*v, k.clone()))
                .collect()
        }
    }

    pub fn show(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext,
        messages: &mut Vec<Message>,
        movie_offset: f64,
    ) -> bool {
        let mut keep_open = true;
        self.hovered_debug_rect = None;

        if self.finding {
            self.generate_results(egui_ctx, context, movie_offset);
        }

        Window::new("Display Object Picker")
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.include_hidden, "Include Hidden");
                    ui.checkbox(&mut self.only_mouse_enabled, "Only Mouse Enabled Objects");
                });
                if self.finding {
                    ui.label("Click somewhere to finish searching");
                } else if ui.button("Start Searching").clicked() {
                    self.finding = true;
                }
                if !self.results.is_empty() {
                    ui.separator();
                    ui.heading("Results");
                    for tree in &self.results {
                        show_object_tree(ui, context, tree, messages, &mut self.hovered_debug_rect);
                    }
                }
            });

        keep_open
    }

    fn generate_results(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext,
        movie_offset: f64,
    ) {
        self.results.clear();
        self.unique_results.clear();

        if let Some(pointer) = egui_ctx.pointer_latest_pos() {
            let pointer = Vec2::new(
                pointer.x * egui_ctx.pixels_per_point(),
                pointer.y * egui_ctx.pixels_per_point(),
            );
            let inverse_view_matrix = context.stage.inverse_view_matrix();
            let pos = inverse_view_matrix
                * Point::from_pixels(pointer.x as f64, pointer.y as f64 - movie_offset);

            let mut results = vec![];
            for child in context.stage.iter_render_list() {
                self.create_result_tree(context, pos, child, &mut results);
            }
            self.results = results;
        }

        if egui_ctx.input_mut(|input| input.pointer.any_click()) {
            self.finding = false;
        }
    }

    fn object_matches(&self, object: DisplayObject, cursor: Point<Twips>) -> bool {
        if self.only_mouse_enabled
            && !object
                .as_interactive()
                .map(|i| i.mouse_enabled())
                .unwrap_or_default()
        {
            return false;
        }
        object.debug_rect_bounds().contains(cursor)
    }

    fn create_result_tree<'gc>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        cursor: Point<Twips>,
        object: DisplayObject<'gc>,
        add_to: &mut Vec<DisplayObjectTree>,
    ) {
        if !self.include_hidden && !object.visible() {
            return;
        }

        if self.object_matches(object, cursor) {
            let handle = DisplayObjectHandle::new(context, object);
            let color =
                DEFAULT_DEBUG_COLORS[self.unique_results.len() % DEFAULT_DEBUG_COLORS.len()];
            let mut tree = DisplayObjectTree {
                handle: handle.clone(),
                children: vec![],
                color,
            };
            self.unique_results.insert(
                handle,
                swf::Color {
                    r: (color[0] * 255.0) as u8,
                    g: (color[1] * 255.0) as u8,
                    b: (color[2] * 255.0) as u8,
                    a: 255,
                },
            );
            if let Some(container) = object.as_container() {
                for child in container.iter_render_list() {
                    self.create_result_tree(context, cursor, child, &mut tree.children);
                }
            }
            add_to.push(tree);
        } else if let Some(container) = object.as_container() {
            for child in container.iter_render_list() {
                self.create_result_tree(context, cursor, child, add_to);
            }
        }
    }
}

fn show_object_tree(
    ui: &mut Ui,
    context: &mut UpdateContext,
    tree: &DisplayObjectTree,
    messages: &mut Vec<Message>,
    hovered_debug_rect: &mut Option<DisplayObjectHandle>,
) {
    if tree.children.is_empty() {
        // This item is not expandable, but we want to keep the space empty where the
        // expand button would be, so it doesn't look like a sibling of the parent.
        ui.indent(ui.id().with(tree.handle.as_ptr()), |ui| {
            show_item(ui, context, tree, messages, hovered_debug_rect);
        });
    } else {
        CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(tree.handle.as_ptr()), true)
            .show_header(ui, |ui| {
                show_item(ui, context, tree, messages, hovered_debug_rect);
            })
            .body(|ui| {
                for child in &tree.children {
                    show_object_tree(ui, context, child, messages, hovered_debug_rect);
                }
            });
    }
}

fn show_item(
    ui: &mut Ui,
    context: &mut UpdateContext,
    tree: &DisplayObjectTree,
    messages: &mut Vec<Message>,
    hovered_debug_rect: &mut Option<DisplayObjectHandle>,
) {
    ui.horizontal(|ui| {
        show_color(
            ui,
            Rgba::from_rgb(tree.color[0], tree.color[1], tree.color[2]),
            Vec2::new(ui.spacing().interact_size.y, ui.spacing().interact_size.y),
        );
        let object = tree.handle.fetch(context.dynamic_root);
        open_display_object_button(
            ui,
            context,
            messages,
            tree.handle.fetch(context.dynamic_root),
            hovered_debug_rect,
        );
        if !object.visible() {
            ui.weak("(Hidden)");
        }
    });
}

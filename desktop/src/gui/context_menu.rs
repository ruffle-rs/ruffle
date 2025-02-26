use crate::custom_event::RuffleEvent;
use egui::{
    vec2, Align, Area, Button, Checkbox, Color32, Frame, Id, Key, KeyboardShortcut, Layout,
    Modifiers, Order, Pos2, Stroke, Style, Widget,
};
use ruffle_core::{ContextMenuItem, PlayerEvent};
use unic_langid::LanguageIdentifier;
use winit::event_loop::EventLoopProxy;

use super::text;

pub struct ContextMenu {
    items: Vec<ContextMenuItem>,
    position: Option<Pos2>,
    close_event: PlayerEvent,
}

impl ContextMenu {
    pub fn new(items: Vec<ContextMenuItem>, close_event: PlayerEvent) -> Self {
        Self {
            items,
            position: None,
            close_event,
        }
    }

    pub fn close_event(self) -> PlayerEvent {
        self.close_event
    }

    pub fn show(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
        event_loop: &EventLoopProxy<RuffleEvent>,
        fullscreen: bool,
    ) -> bool {
        let mut item_clicked = false;
        self.position = self.position.or(egui_ctx.pointer_latest_pos());

        let area = Area::new(Id::new("context_menu"))
            .order(Order::Foreground)
            .fixed_pos(self.position.unwrap_or_default())
            .constrain_to(egui_ctx.screen_rect())
            .interactable(true)
            .show(egui_ctx, |ui| {
                set_menu_style(ui.style_mut());
                Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_max_width(150.0);
                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                        for (i, item) in self.items.iter().enumerate() {
                            if i != 0 && item.separator_before {
                                ui.separator();
                            }
                            let clicked = if item.checked {
                                Checkbox::new(&mut true, &item.caption).ui(ui).clicked()
                            } else {
                                let button = Button::new(&item.caption)
                                    .wrap_mode(egui::TextWrapMode::Extend);

                                ui.add_enabled(item.enabled, button).clicked()
                            };
                            if clicked {
                                let _ =
                                    event_loop.send_event(RuffleEvent::ContextMenuItemClicked(i));
                                item_clicked = true;
                            }
                        }

                        if fullscreen {
                            ui.separator();
                            if Button::new(text(locale, "context-menu-exit-fullscreen"))
                                .shortcut_text(ui.ctx().format_shortcut(&KeyboardShortcut::new(
                                    Modifiers::NONE,
                                    Key::Escape,
                                )))
                                .wrap_mode(egui::TextWrapMode::Extend)
                                .ui(ui)
                                .clicked()
                            {
                                let _ = event_loop.send_event(RuffleEvent::ExitFullScreen);
                                item_clicked = true;
                            }
                        }
                    })
                })
            });

        let should_close = item_clicked
            || area.response.clicked_elsewhere()
            || egui_ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));

        !should_close
    }
}

// Shamelessly stolen from egui menu::set_menu_style, a private internal function
fn set_menu_style(style: &mut Style) {
    style.spacing.button_padding = vec2(2.0, 0.0);
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
}

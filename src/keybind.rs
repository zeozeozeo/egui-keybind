use crate::Bind;
use egui::{
    pos2, vec2, Event, Id, Key, KeyboardShortcut, ModifierNames, PointerButton, RichText, Sense,
    TextStyle, Ui, Widget, WidgetInfo, WidgetText, WidgetType,
};

/// A keybind (hotkey) widget for [egui].
pub struct Keybind<'a, B: Bind> {
    bind: &'a mut B,
    reset: B,
    text: &'a str,
    id: Id,
    reset_key: Option<Key>,
    modifier_names: &'a ModifierNames<'a>,
}

impl<'a, B: Bind> Keybind<'a, B> {
    /// Create a new [Keybind] for a given [Bind].
    ///
    /// # Arguments
    ///
    /// * `bind` - The bind to use for the [Keybind].
    /// * `id` - ID for the [Keybind] in [egui]'s memory.
    pub fn new(bind: &'a mut B, id: impl Into<Id>) -> Self {
        let prev_bind = bind.clone();
        Self {
            bind,
            reset: prev_bind,
            text: "",
            id: id.into(),
            reset_key: None,
            modifier_names: &ModifierNames::NAMES,
        }
    }

    /// Set the text of the [Keybind]. This will be displayed next to the
    /// keybind widget (and used for accessibility).
    ///
    /// You can remove the text by setting it to an empty string.
    /// By default there is no text.
    pub fn with_text(mut self, text: &'a str) -> Self {
        self.text = text;
        self
    }

    /// Set the bind of the [Keybind].
    ///
    /// By default this is the bind that was passed to `new`.
    pub fn with_bind(mut self, bind: &'a mut B) -> Self {
        self.bind = bind;
        self
    }

    /// Set the ID of the [Keybind] in [egui]'s memory.
    ///
    /// By default this is the ID that was passed in `new`.
    pub fn with_id(mut self, id: impl Into<Id>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the key that resets the [Keybind]. If [None], the [Keybind] will
    /// never reset to its' previous value.
    ///
    /// By default this is [None].
    pub fn with_reset_key(mut self, key: Option<Key>) -> Self {
        self.reset_key = key;
        self
    }

    /// Set the bind that the [Keybind] will reset to after the reset key gets pressed.
    ///
    /// By default this is the same as the bind passed to `new`.
    pub fn with_reset(mut self, prev_bind: B) -> Self {
        self.reset = prev_bind;
        self
    }

    /// Set the modifier names to use for the [Keybind]. By default this is [`ModifierNames::NAMES`].
    pub fn with_modifier_names(mut self, modifier_names: &'a ModifierNames<'a>) -> Self {
        self.modifier_names = modifier_names;
        self
    }
}

/// Get the widget expecting value from egui's memory.
fn get_expecting(ui: &Ui, id: Id) -> bool {
    let expecting = ui.ctx().memory_mut(|memory| {
        *memory
            .data
            .get_temp_mut_or_default::<bool>(ui.make_persistent_id(id))
    });
    expecting
}

/// Set the widget expecting value in egui's memory.
fn set_expecting(ui: &Ui, id: Id, expecting: bool) {
    ui.ctx().memory_mut(|memory| {
        *memory
            .data
            .get_temp_mut_or_default(ui.make_persistent_id(id)) = expecting;
    });
}

impl<'a, B: Bind> Widget for Keybind<'a, B> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let text = self.bind.format(self.modifier_names, false);

        let galley = WidgetText::RichText(RichText::new(text.clone())).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            0.0,
            TextStyle::Button,
        );

        let size = ui.spacing().interact_size.max(galley.size());
        let button_padding = ui.spacing().button_padding;
        let mut widget_size = size + button_padding * vec2(2.0, 1.0);

        // compute the text galley next to the widget (set by with_text), expand
        // widget appropriately
        let text_galley = if !self.text.is_empty() {
            let galley = WidgetText::RichText(RichText::new(self.text)).into_galley(
                ui,
                None,
                ui.available_width() - widget_size.x, // not exactly right
                TextStyle::Button,
            );
            Some(galley)
        } else {
            None
        };

        let custom_text_width = text_galley.clone().map_or(0.0, |text_galley| {
            ui.spacing().icon_spacing + text_galley.size().x
        });
        widget_size.x += custom_text_width;

        let (rect, mut response) = ui.allocate_exact_size(widget_size, Sense::click());

        // calculate size of the widget without the custom text
        let mut hotkey_rect = rect;
        *hotkey_rect.right_mut() -= custom_text_width;

        // see if we're currently waiting for any key (pull from egui's memory)
        let mut expecting = get_expecting(ui, self.id);
        let prev_expecting = expecting;
        if response.clicked() {
            expecting = !expecting;
        }

        // add widget info for accessibility. this generates a string like "Ctrl+T. Open the terminal"
        // if the keybind was created with `with_text`
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::Button,
                expecting,
                expecting,
                if self.text.is_empty() {
                    text.clone() // just read out the hotkey
                } else {
                    text.clone() + ". " + self.text
                },
            )
        });

        if expecting {
            if response.clicked_elsewhere() {
                // the user has clicked somewhere else, stop capturing input
                expecting = false;
            } else {
                // everything ok, capture keyboard input
                let kb = ui.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            repeat: false,
                            ..
                        } => Some((*key, *modifiers)),
                        _ => None,
                    })
                });

                // capture mouse input
                let pointer = ui.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        Event::PointerButton {
                            button,
                            pressed: true,
                            ..
                        } if *button != PointerButton::Primary
                            && *button != PointerButton::Secondary =>
                        {
                            Some(*button)
                        }
                        _ => None,
                    })
                });

                // set keybind
                if kb.is_some() || pointer.is_some() {
                    self.bind
                        .set(kb.map(|kb| KeyboardShortcut::new(kb.1, kb.0)), pointer);
                    response.mark_changed();
                    expecting = false;
                }
            }

            if let Some(reset_key) = self.reset_key {
                // the reset key was pressed
                if ui.input(|i| i.key_pressed(reset_key)) {
                    *self.bind = self.reset;
                    expecting = false;
                    response.mark_changed();
                }
            }
        }

        // paint
        if ui.is_rect_visible(rect) {
            // paint bg rect
            let visuals = ui.style().interact_selectable(&response, expecting);
            ui.painter().rect(
                hotkey_rect.expand(visuals.expansion),
                visuals.rounding,
                visuals.bg_fill,
                visuals.bg_stroke,
            );

            // align text to center in rect that is shrinked to match button padding
            let mut text_pos = ui
                .layout()
                .align_size_within_rect(galley.size(), hotkey_rect.shrink2(button_padding))
                .min;

            // align text to center of the button if it doesn't expand the rect
            if text_pos.x + galley.size().x + button_padding.x < hotkey_rect.right() {
                text_pos.x += hotkey_rect.size().x / 2.0 - galley.size().x / 2.0 - button_padding.x;
            }

            // paint text inside button
            ui.painter().galley(text_pos, galley, visuals.text_color());

            // paint galley for text outside on the left, if any
            if let Some(text_galley) = text_galley {
                let text_pos = pos2(
                    hotkey_rect.right() + ui.spacing().icon_spacing,
                    hotkey_rect.center().y - 0.5 * text_galley.size().y,
                );
                ui.painter().galley(
                    text_pos,
                    text_galley,
                    ui.style().noninteractive().text_color(),
                );
            }
        }

        if prev_expecting != expecting {
            set_expecting(ui, self.id, expecting);
        }
        response
    }
}

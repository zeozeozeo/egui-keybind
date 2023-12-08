use crate::Bind;
use egui::{
    vec2, Event, Id, Key, KeyboardShortcut, ModifierNames, PointerButton, RichText, Sense,
    TextStyle, Ui, Widget, WidgetInfo, WidgetText, WidgetType,
};

/// A keybind (hotkey) widget for [egui].
pub struct Keybind<'a, B: Bind> {
    bind: &'a mut B,
    prev_bind: B,
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
            prev_bind,
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
    pub fn with_text(&mut self, text: &'a str) -> &mut Self {
        self.text = text;
        self
    }

    /// Set the bind of the [Keybind].
    ///
    /// By default this is the bind that was passed to `new`.
    pub fn with_bind(&mut self, bind: &'a mut B) -> &mut Self {
        self.bind = bind;
        self
    }

    /// Set the ID of the [Keybind] in [egui]'s memory.
    ///
    /// By default this is the ID that was passed in `new`.
    pub fn with_id(&mut self, id: impl Into<Id>) -> &mut Self {
        self.id = id.into();
        self
    }

    /// Set the key that resets the [Keybind]. If [None], the [Keybind] will
    /// never reset to its' previous value.
    ///
    /// By default this is [None].
    pub fn with_reset_key(&mut self, key: Option<Key>) -> &mut Self {
        self.reset_key = key;
        self
    }

    /// Set the bind that the [Keybind] will reset to after the reset key gets pressed.
    ///
    /// By default this is the same as the bind passed to `new`.
    pub fn with_prev_bind(&mut self, prev_bind: B) -> &mut Self {
        self.prev_bind = prev_bind;
        self
    }

    /// Set the modifier names to use for the [Keybind]. By default this is [`ModifierNames::NAMES`].
    pub fn with_modifier_names(&mut self, modifier_names: &'a ModifierNames<'a>) -> &mut Self {
        self.modifier_names = modifier_names;
        self
    }
}

fn get_expecting(ui: &Ui, id: Id) -> bool {
    let expecting = ui.ctx().memory_mut(|memory| {
        *memory
            .data
            .get_temp_mut_or_default::<bool>(ui.make_persistent_id(id))
    });
    expecting
}

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
            Some(false),
            0.0,
            TextStyle::Button,
        );

        let size = ui.spacing().interact_size.max(galley.size());
        let button_padding = ui.spacing().button_padding;
        let (rect, mut response) =
            ui.allocate_exact_size(size + button_padding * vec2(2.0, 1.0), Sense::click());

        // add widget info for accessibility. this generates a string like "Ctrl+T. Open the terminal"
        // if the keybind was created with `with_text`
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, false, text.clone() + ". " + self.text)
        });

        // see if we're currently waiting for any key (pull from egui's memory)
        let mut expecting = get_expecting(ui, self.id);
        let prev_expecting = expecting;
        if response.clicked() {
            expecting = !expecting;
        }

        if expecting {
            if response.clicked_elsewhere() {
                // the user has clicked somewhere else, stop capturing input
                expecting = false;
            } else if let Some(reset_key) = self.reset_key {
                // the reset key was pressed
                if ui.input(|i| i.key_pressed(reset_key)) {
                    *self.bind = self.prev_bind;
                    expecting = false;
                    response.mark_changed();
                }
            } else {
                // everything ok, capture keyboard input
                let kb = ui.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            repeat: false,
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
        }

        // paint
        if ui.is_rect_visible(rect) {
            // paint bg rect
            let visuals = ui.style().interact_selectable(&response, expecting);
            ui.painter().rect(
                rect.expand(visuals.expansion),
                visuals.rounding,
                visuals.bg_fill,
                visuals.bg_stroke,
            );

            // align text to center in rect that is shrinked to match button padding
            let mut text_pos = ui
                .layout()
                .align_size_within_rect(galley.size(), rect.shrink2(button_padding))
                .min;

            // align button to center if it doesn't expand the rect
            if text_pos.x + galley.size().x + button_padding.x < rect.right() {
                text_pos.x += rect.size().x / 2.0 - galley.size().x / 2.0 - button_padding.x;
            }

            // paint text
            galley.paint_with_visuals(ui.painter(), text_pos, &visuals);
        }

        if prev_expecting != expecting {
            set_expecting(ui, self.id, expecting);
        }
        response
    }
}

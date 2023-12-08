use egui::{Key, KeyboardShortcut, ModifierNames, PointerButton};

/// A trait can can be used for keybindings.
///
/// Must have a function to update the keybinding with a given [Key] and
/// [Modifiers], aswell as a method that formats the keybinding as a [String].
///
/// Must implement [Clone].
pub trait Bind: Clone {
    /// Set the keybind with a given [KeyboardShortcut] and/or [PointerButton].
    ///
    /// # Arguments
    /// * `keyboard` - The keyboard shortcut to set ([KeyboardShortcut]), or [None].
    /// * `pointer` - The pointer button to set ([PointerButton]), or [None].
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, pointer: Option<PointerButton>);

    /// Format the current keybind as a [String].
    ///
    /// # Arguments
    /// * `names` - The [ModifierNames] to use.
    /// * `is_mac` - Whether to use MacOS symbols.
    ///
    /// # Returns
    /// The formatted keybind as a [String].
    fn format(&self, names: &ModifierNames<'_>, is_mac: bool) -> String;
}

/// A [Bind] implementation for [egui]'s [KeyboardShortcut].
impl Bind for KeyboardShortcut {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, _pointer: Option<PointerButton>) {
        if let Some(keyboard) = keyboard {
            *self = keyboard
        }
    }

    fn format(&self, names: &ModifierNames<'_>, is_mac: bool) -> String {
        self.format(names, is_mac)
    }
}

impl Bind for Option<KeyboardShortcut> {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, _pointer: Option<PointerButton>) {
        *self = keyboard;
    }

    fn format(&self, names: &ModifierNames<'_>, is_mac: bool) -> String {
        self.as_ref().map_or_else(
            || "None".to_string(),
            |shortcut| shortcut.format(names, is_mac),
        )
    }
}

/// A [Bind] implementation for [egui]'s [Key]. Ignores modifiers.
impl Bind for Key {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, _pointer: Option<PointerButton>) {
        if let Some(keyboard) = keyboard {
            *self = keyboard.key
        }
    }

    fn format(&self, _names: &ModifierNames<'_>, _is_mac: bool) -> String {
        self.name().to_string()
    }
}

impl Bind for Option<Key> {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, _pointer: Option<PointerButton>) {
        if let Some(keyboard) = keyboard {
            *self = Some(keyboard.key)
        }
    }

    fn format(&self, _names: &ModifierNames<'_>, _is_mac: bool) -> String {
        self.as_ref()
            .map_or_else(|| "None".to_string(), |key| key.name().to_string())
    }
}

/// A [Bind] implementation for [egui]'s [PointerButton]. Ignores keys and modifiers.
impl Bind for PointerButton {
    fn set(&mut self, _keyboard: Option<KeyboardShortcut>, pointer: Option<PointerButton>) {
        if let Some(pointer) = pointer {
            *self = pointer
        }
    }

    fn format(&self, _names: &ModifierNames<'_>, _is_mac: bool) -> String {
        format!("{:?}", self)
    }
}

impl Bind for Option<PointerButton> {
    fn set(&mut self, _keyboard: Option<KeyboardShortcut>, pointer: Option<PointerButton>) {
        *self = pointer;
    }

    fn format(&self, _names: &ModifierNames<'_>, _is_mac: bool) -> String {
        self.as_ref()
            .map_or_else(|| "None".to_string(), |button| format!("{:?}", button))
    }
}

/// A keybind that can be set with either the keyboard or a mouse.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Shortcut {
    /// Keyboard shortcut, if any. This can be set along with the mouse shortcut.
    pub keyboard: Option<KeyboardShortcut>,
    /// Mouse button, if any. This can be set along with the keyboard shortcut.
    pub pointer: Option<PointerButton>,
}

impl Shortcut {
    /// No keybind.
    pub const NONE: Self = Self {
        keyboard: None,
        pointer: None,
    };

    /// Create a new [Shortcut].
    ///
    /// # Arguments
    ///
    /// * `keyboard` - The keyboard shortcut to set ([KeyboardShortcut]), or [None].
    /// * `pointer` - The pointer button to set ([PointerButton]), or [None].
    pub fn new(keyboard: Option<KeyboardShortcut>, pointer: Option<PointerButton>) -> Self {
        Self { keyboard, pointer }
    }

    /// Check if the keybind is pressed. If it is, [egui] will consume the pressed keys so it
    /// doesn't trigger the next frame.
    pub fn consume(&self, input: &mut egui::InputState) -> bool {
        let mut pressed = false;
        if let Some(kb) = &self.keyboard {
            pressed = input.consume_shortcut(kb);
        }
        if let Some(button) = self.pointer {
            if self.keyboard.is_none() {
                return input.pointer.button_clicked(button);
            }
            pressed &= input.pointer.button_clicked(button);
        }
        pressed
    }
}

impl Bind for Shortcut {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, pointer: Option<PointerButton>) {
        self.keyboard = keyboard;
        self.pointer = pointer;
    }

    fn format(&self, names: &ModifierNames<'_>, is_mac: bool) -> String {
        let mut string = self
            .keyboard
            .map_or_else(|| String::with_capacity(9), |kb| kb.format(names, is_mac));
        if let Some(pointer) = self.pointer {
            if !string.is_empty() {
                string.push('+');
            }
            string.push_str(&pointer.format(names, is_mac));
        }
        if string.is_empty() {
            string.push_str("None");
        }
        string
    }
}

impl From<Shortcut> for Option<KeyboardShortcut> {
    fn from(value: Shortcut) -> Self {
        value.keyboard
    }
}

impl From<Shortcut> for Option<PointerButton> {
    fn from(value: Shortcut) -> Self {
        value.pointer
    }
}

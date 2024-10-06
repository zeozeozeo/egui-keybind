use egui::{InputState, Key, KeyboardShortcut, ModifierNames, PointerButton};

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

    /// Check if the keybind is pressed.
    ///
    /// # Arguments
    /// * `input` - The [InputState] to check with.
    ///
    /// # Returns
    /// Whether the keybind is pressed.
    fn pressed(&self, input: &mut InputState) -> bool;
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

    fn pressed(&self, input: &mut InputState) -> bool {
        input.consume_shortcut(self)
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

    fn pressed(&self, input: &mut InputState) -> bool {
        if let Some(shortcut) = self {
            input.consume_shortcut(shortcut)
        } else {
            false
        }
    }
}

/// A [Bind] implementation for [egui]'s [Key]. Ignores modifiers.
impl Bind for Key {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, _pointer: Option<PointerButton>) {
        if let Some(keyboard) = keyboard {
            *self = keyboard.logical_key
        }
    }

    fn format(&self, _names: &ModifierNames<'_>, _is_mac: bool) -> String {
        self.name().to_string()
    }

    fn pressed(&self, input: &mut InputState) -> bool {
        input.key_pressed(*self)
    }
}

impl Bind for Option<Key> {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, _pointer: Option<PointerButton>) {
        if let Some(keyboard) = keyboard {
            *self = Some(keyboard.logical_key)
        }
    }

    fn format(&self, _names: &ModifierNames<'_>, _is_mac: bool) -> String {
        self.as_ref()
            .map_or_else(|| "None".to_string(), |key| key.name().to_string())
    }

    fn pressed(&self, input: &mut InputState) -> bool {
        if let Some(key) = self {
            input.key_pressed(*key)
        } else {
            false
        }
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

    fn pressed(&self, input: &mut InputState) -> bool {
        input.pointer.button_pressed(*self)
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

    fn pressed(&self, input: &mut InputState) -> bool {
        if let Some(button) = self {
            input.pointer.button_pressed(*button)
        } else {
            false
        }
    }
}

/// A keybind that can be set with either the keyboard or a mouse.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Shortcut {
    /// Keyboard shortcut, if any. This can be set along with the mouse shortcut.
    keyboard: Option<KeyboardShortcut>,
    /// Mouse button, if any. This can be set along with the keyboard shortcut.
    pointer: Option<PointerButton>,
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
        Self {
            keyboard,
            pointer,
        }
    }

    /// Keyboard shortcut, if any. This can be set along with the mouse shortcut.
    #[inline]
    pub fn keyboard(&self) -> Option<KeyboardShortcut> {
        self.keyboard
    }

    /// Mouse button, if any. This can be set along with the keyboard shortcut.
    #[inline]
    pub const fn pointer(&self) -> Option<PointerButton> {
        self.pointer
    }
}

impl Bind for Shortcut {
    fn set(&mut self, keyboard: Option<KeyboardShortcut>, pointer: Option<PointerButton>) {
        self.keyboard = keyboard;
        self.pointer = pointer;
    }

    fn format(&self, names: &ModifierNames<'_>, is_mac: bool) -> String {
        let mut string = self.keyboard.map_or_else(
            || String::with_capacity(9),
            |kb| Into::<KeyboardShortcut>::into(kb).format(names, is_mac),
        );
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

    fn pressed(&self, input: &mut InputState) -> bool {
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

use egui_keybind::{Bind, Keybind, Shortcut};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-keybind example",
        options,
        Box::new(|_cc| Box::<ExampleApp>::default()),
    )
}

struct ExampleApp {
    shortcut: Shortcut,
    mouse_shortcut: Option<egui::PointerButton>,
    default_shortcut: Shortcut,
    x_reset_shortcut: Shortcut,
    times_pressed: usize,
}

impl Default for ExampleApp {
    fn default() -> Self {
        Self {
            shortcut: Shortcut::NONE,
            mouse_shortcut: None,
            default_shortcut: Shortcut::new(
                Some(egui::KeyboardShortcut::new(
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                    egui::Key::D,
                )),
                None,
            ),
            x_reset_shortcut: Shortcut::NONE,
            times_pressed: 0,
        }
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_example(ctx, ui);
            });
        });
    }
}

impl ExampleApp {
    fn show_example(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("egui-keybind example");

        ui.add_space(4.0);
        ui.label("A simple keybind:");
        let response = ui.add(Keybind::new(&mut self.shortcut, "example_keybind"));
        if response.changed() {
            self.times_pressed = 0;
        }

        ui.separator();
        ui.label("A keybind that only accepts extra mouse buttons:");
        ui.add(Keybind::new(
            &mut self.mouse_shortcut,
            "example_mouse_keybind",
        ));

        ui.separator();
        ui.label("A keybind that is Ctrl+Shift+D by default:");
        ui.add(
            Keybind::new(&mut self.default_shortcut, "default_keybind")
                .with_text("I have text too"),
        );

        ui.separator();
        ui.label("A keybind that resets to X when Escape is pressed:");
        ui.add(
            Keybind::new(&mut self.x_reset_shortcut, "x_reset_shortcut")
                .with_text("Click me and press Esc!")
                .with_reset(Shortcut::new(
                    Some(egui::KeyboardShortcut {
                        modifiers: egui::Modifiers::NONE,
                        logical_key: egui::Key::X,
                    }),
                    None,
                ))
                .with_reset_key(Some(egui::Key::Escape)),
        );

        ui.separator();

        // display keybind text
        let keybind_text = self.shortcut.format(&egui::ModifierNames::NAMES, false);
        ui.label(format!(
            "First keybind: {keybind_text} (you can use modifier keys!)"
        ));

        if ctx.input_mut(|i| self.shortcut.pressed(i)) {
            self.times_pressed += 1;
        }
        if keybind_text != "None" {
            ui.label(format!(
                "{keybind_text} has been pressed {} times",
                self.times_pressed
            ));
        }

        // reset all keybinds
        if ui.button("Reset all").clicked() {
            *self = Self::default();
        }
    }
}

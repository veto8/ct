#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        //initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "CT",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(CT::default())
        }),
    )
}

#[derive(Default)]
struct CT {
    label: String,
    counter: usize,

    text: String,
    picked_path: String,
    cursor1: usize,
    cursor2: usize,
    password: String,
    window_help_open: bool,
    window_about_open: bool,
}

impl eframe::App for CT {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            let _password = ui.add(
                egui::TextEdit::singleline(&mut self.password)
                    .hint_text("Please enter your password")
                    .desired_width(f32::INFINITY)
                    .password(true),
            );

            let _scroll = egui::ScrollArea::vertical().show(ui, |ui| {
                let textedit = egui::TextEdit::multiline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Please enter your text");

                let response = ui.add_sized(ui.available_size(), textedit);
                //https://docs.rs/egui/0.21.0/egui/struct.Response.html#method.hovered
                let resp_id = response.id;
                if let Some(state) = egui::TextEdit::load_state(ui.ctx(), resp_id) {
                    if let Some(ccursor) = state.ccursor_range() {
                        self.cursor1 = ccursor.secondary.index;
                        self.cursor2 = ccursor.primary.index;
                    }
                }
            });
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Crypt Text").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Open Crypt Textx").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });
    }
}

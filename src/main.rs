#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate ct_nox;
use ct::icon::get_icon;
use ct_nox::ct_nox::{read_file, write_file};
use ct_nox::decrypt::decrypt;
use ct_nox::encrypt::encrypt;

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use eframe::egui::TextBuffer as _;
use eframe::egui::Vec2;
use eframe::egui::containers::popup;
use rfd::FileDialog;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(900.0, 700.0)), // Set initial window size here
        ..Default::default()
    };
    eframe::run_native(
        "CT",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_pixels_per_point(2.5);
            Box::new(CT::default())
        }),
    )
}

#[derive(Default)]
struct CT {
    text: String,
    picked_path: String,
    cursor1: usize,
    cursor2: usize,
    password: String,
    search: String,
    window_help_open: bool,
    window_about_open: bool,
}

impl eframe::App for CT {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            let _password = ui.add(
                egui::TextEdit::singleline(&mut self.password)
                    .hint_text("Password")
                    .desired_width(f32::INFINITY)
                    .password(true),
            );
            ui.horizontal(|ui| {
                let spacing = ui.spacing().item_spacing.x;
                let available_width = ui.available_width() - (spacing * 2.0);
                let button_width = (available_width / 100.00) * 15.00;
                let search_width = (available_width / 100.00) * 85.00;
                let button_height = 20.0;
                let button_size = egui::Vec2::new(button_width, button_height);

                let _search = ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search")
                        .desired_width(search_width)
                        .password(false),
                );

                if ui
                    .add(egui::Button::new("Search").min_size(button_size))
                    .clicked()
                {}
            });

            ui.horizontal(|ui| {
                let num_buttons = 6.0;
                let spacing = ui.spacing().item_spacing.x;
                let total_spacing = spacing * (num_buttons - 1.0);

                let available_width = ui.available_width();
                let button_width = (available_width - total_spacing) / num_buttons;
                let button_height = 20.0;
                let button_size = egui::Vec2::new(button_width, button_height);

                if ui
                    .add(egui::Button::new("Open").min_size(button_size))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.picked_path = path.display().to_string();
                        let ct = read_file(&self.picked_path.clone());
                        self.text = decrypt(&ct, &self.password);
                    }
                }

                if ui
                    .add(egui::Button::new("Save").min_size(button_size))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.picked_path = path.display().to_string();
                        println!("save crypt text to: {}", self.picked_path);
                        let ct = encrypt(&self.text, &self.password);
                        let _x = write_file(&self.picked_path.clone(), &ct);
                    }
                }

                if ui
                    .add(egui::Button::new("Cut").min_size(button_size))
                    .clicked()
                {
                    let r = get_char_range(self.cursor1, self.cursor2);
                    let st = self.text.char_range(r.clone());
                    ui.output_mut(|o| o.copied_text = st.to_string());
                    self.text.delete_char_range(r.clone());
                }
                if ui
                    .add(egui::Button::new("Copy").min_size(button_size))
                    .clicked()
                {
                    let r = get_char_range(self.cursor1, self.cursor2);
                    let st = self.text.char_range(r.clone());
                    ui.output_mut(|o| o.copied_text = st.to_string());
                }
                if ui
                    .add(egui::Button::new("Paste").min_size(button_size))
                    .clicked()
                {
                    let txt = cli_clipboard::get_contents().unwrap();
                    let r = get_char_range(self.cursor1, self.cursor2);
                    self.text.insert_text(&txt, r.start);
                }
                if ui
                    .add(egui::Button::new("Close").min_size(button_size))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

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

fn get_char_range(c1: usize, c2: usize) -> std::ops::Range<usize> {
    //https://docs.rs/egui/latest/egui/widgets/text_edit/trait.TextBuffer.html#method.char_range
    let mut a = c1;
    let mut b = c2;
    if a > b {
        a = c2;
        b = c1;
    }
    let r = std::ops::Range { start: a, end: b };
    return r;
}

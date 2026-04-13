#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate ct_nox;
use ct::icon::get_icon;
use ct_nox::ct_nox::{read_file, write_file};
use ct_nox::decrypt::decrypt;
use ct_nox::encrypt::encrypt;
use eframe::egui;
use eframe::egui::TextBuffer;
use eframe::egui::{IconData, Pos2, Vec2};

use i18n_embed::LanguageLoader;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "locales/"] // Path to the compiled localization resources
struct Asset;

fn main() -> Result<(), eframe::Error> {
    let (icon_rgba, icon_width, icon_height) = {
        let rgba = get_icon();
        (rgba, 64, 64)
    };

    let icon_data = IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    };

    //let icon_data = get_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(900.0, 750.0))
            .with_icon(icon_data),
        //icon_data: Some(load_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "CT",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_pixels_per_point(2.2);
            Box::new(CT::default())
        }),
    )
}

#[derive(Default)]
struct CT {
    text: String,
    picked_path: String,
    status_text: String,
    cursor1: usize,
    cursor2: usize,
    password: String,
    search: String,
    hide_password: bool,
    search_bar: bool,
    show_popup: bool,
    popup_position: Pos2,
}

impl eframe::App for CT {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                let spacing = ui.spacing().item_spacing.x;
                let available_width = ui.available_width() - (spacing * 2.0);
                let button_width = (available_width / 100.00) * 25.00;
                let password_width = (available_width / 100.00) * 75.00;
                let button_height = 20.0;
                let button_size = egui::Vec2::new(button_width, button_height);

                let _password = ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .hint_text("Password")
                        .desired_width(password_width)
                        .password(!self.hide_password),
                );

                let button_text = if self.hide_password {
                    "Hide Password"
                } else {
                    "Show Password"
                };
                if ui
                    .add(egui::Button::new(button_text).min_size(button_size))
                    .clicked()
                {
                    self.hide_password = !self.hide_password;
                }
            });

            ui.horizontal(|ui| {
                let num_buttons = 7.0;
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
                    .add(egui::Button::new("Search").min_size(button_size))
                    .clicked()
                {
                    self.search_bar = !self.search_bar;
                }

                if ui
                    .add(egui::Button::new("Close").min_size(button_size))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            if self.search_bar {
                ui.horizontal(|ui| {
                    let _search = ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Search")
                            .desired_width(f32::INFINITY),
                    );
                });
            }

            ui.add_space(2.0);
            let _scroll = egui::ScrollArea::vertical().show(ui, |ui| {
                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let mut layout_job = egui::text::LayoutJob::default();
                    let target_word: &str = self.search.as_str();
                    if target_word != ""
                        && let Some(pos) = string.find(target_word)
                    {
                        layout_job.append(&string[..pos], 0.0, egui::TextFormat::default());

                        let red_color = egui::Color32::RED;
                        let color_format = egui::TextFormat {
                            color: red_color,
                            ..Default::default()
                        };
                        layout_job.append(&string[pos..pos + target_word.len()], 0.0, color_format);

                        layout_job.append(
                            &string[pos + target_word.len()..],
                            0.0,
                            egui::TextFormat::default(),
                        );
                    } else {
                        layout_job.append(string, 0.0, egui::TextFormat::default());
                    }

                    layout_job.wrap.max_width = wrap_width;

                    ui.fonts(|f| f.layout_job(layout_job))
                };

                let textedit = egui::TextEdit::multiline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Please enter your text")
                    .layouter(&mut layouter);
                let response = ui.add_sized(ui.available_size(), textedit);
                //https://docs.rs/egui/0.21.0/egui/struct.Response.html#method.hovered
                let resp_id = response.id;

                if let Some(state) = egui::TextEdit::load_state(ui.ctx(), resp_id) {
                    if let Some(ccursor) = state.ccursor_range() {
                        //if let Some(ccursor) = self.cursor.char_range() {
                        self.cursor1 = ccursor.secondary.index;
                        self.cursor2 = ccursor.primary.index;
                    }
                }

                if response.clicked_by(egui::PointerButton::Secondary) {
                    self.show_popup = true;
                    self.popup_position = response.interact_pointer_pos().unwrap_or(Pos2::ZERO); // Store click position
                }
            });
        });

        if self.show_popup {
            let popup_id = egui::Id::new("my_popup");
            egui::Area::new(popup_id)
                .fixed_pos(self.popup_position)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button("Copy").clicked() {
                            let r = get_char_range(self.cursor1, self.cursor2);
                            let st = self.text.char_range(r.clone());
                            ui.output_mut(|o| o.copied_text = st.to_string());
                            self.show_popup = false;
                        }
                        if ui.button("Cut").clicked() {
                            let r = get_char_range(self.cursor1, self.cursor2);
                            let st = self.text.char_range(r.clone());
                            ui.output_mut(|o| o.copied_text = st.to_string());
                            self.text.delete_char_range(r.clone());
                            self.show_popup = false;
                        }

                        if ui.button("Paste").clicked() {
                            let txt = cli_clipboard::get_contents().unwrap();
                            let r = get_char_range(self.cursor1, self.cursor2);
                            self.text.insert_text(&txt, r.start);
                            self.show_popup = false;
                        }
                        if ui.button("Close").clicked() {
                            self.show_popup = false;
                        }
                    });

                    if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                        self.show_popup = false;
                    }
                });
        }

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

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Status: {}", self.status_text));
                ui.separator();
                //ui.add(egui::ProgressBar::new(self.progress).show_percentage());
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

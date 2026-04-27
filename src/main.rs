#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate ct_nox;
use ct::config::get_config;
use ct::icon::get_icon;
use ct_nox::ct_nox::{read_file, write_file};
use ct_nox::decrypt::decrypt;
use ct_nox::encrypt::encrypt;
use eframe::egui;
use eframe::egui::TextBuffer;
use eframe::egui::{ComboBox, IconData, Pos2, Vec2};
use i18n_embed::{
    DesktopLanguageRequester,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use i18n_embed_fl::fl;
// use libs::config::get_config;
use egui::{Context, FontDefinitions};
use rust_embed::RustEmbed;
use std::ops::Range; // Make sure to import this
#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

fn main() -> Result<(), eframe::Error> {
    let config = get_config();

    let loader: FluentLanguageLoader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(&loader, &Localizations, &requested_languages);
    println!("{:?}", _result);

    let x = fl!(loader, "open");
    println!("{:?}", x);
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
            CT::configure_egui_fonts(&cc.egui_ctx); // ← Add this line

            Box::new(CT::default())
        }),
    )
}

struct CT {
    loader: FluentLanguageLoader,
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
    st: String,
    r: Range<usize>,
    panel_central: bool,
    panel_setting: bool,
    selected_language: String,
    languages: Vec<String>,
}

//    fn new(_cc: &eframe::CreationContext<'_>) -> Self {

impl CT {
    pub fn configure_egui_fonts(ctx: &Context) {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "noto_sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/noto-sans.ttf")),
        );

        fonts.font_data.insert(
            "noto_sans_cjk".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/noto-sans-cjk.otf")),
        );

        fonts.font_data.insert(
            "thai".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/thai.ttf")),
        );

        fonts.font_data.insert(
            "ethiopic".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/ethiopic.ttf")),
        );

        fonts.font_data.insert(
            "arabic".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/arabic.ttf")),
        );

        fonts.font_data.insert(
            "armenian".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/armenian.ttf")),
        );

        fonts.font_data.insert(
            "bengali".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/bengali.ttf")),
        );
        fonts.font_data.insert(
            "georgian".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/georgian.ttf")),
        );
        fonts.font_data.insert(
            "gujarati".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/gujarati.ttf")),
        );
        fonts.font_data.insert(
            "kannada".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/kannada.ttf")),
        );
        fonts.font_data.insert(
            "khmer".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/khmer.ttf")),
        );

        fonts.font_data.insert(
            "lao".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/lao.ttf")),
        );

        fonts.font_data.insert(
            "myammar".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/myammar.ttf")),
        );

        fonts.font_data.insert(
            "malayalam".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/malayalam.ttf")),
        );

        fonts.font_data.insert(
            "gurmukhi".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/gurmukhi.ttf")),
        );

        fonts.font_data.insert(
            "sinhala".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/sinhala.tff")),
        );

        fonts.font_data.insert(
            "tamil".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/tamil.ttf")),
        );

        fonts.font_data.insert(
            "telugu".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/telugu.ttf")),
        );

        fonts.font_data.insert(
            "hebrew".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/hebrew.ttf")),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_sans".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(1, "noto_sans_cjk".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(2, "thai".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(3, "ethiopic".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(4, "arabic".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(5, "armenian".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(6, "bengali".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(7, "georgian".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(8, "gujarati".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(9, "kannada".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(10, "khmer".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(11, "lao".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(12, "myammar".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(13, "malayalam".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(14, "sinhala".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(15, "tamil".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(16, "telugu".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(17, "hebrew".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(17, "gurmukhi".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "noto_sans".to_owned());

        ctx.set_fonts(fonts);
    }
}

impl Default for CT {
    fn default() -> Self {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        let requested_languages = DesktopLanguageRequester::requested_languages();
        let _result = i18n_embed::select(&loader, &Localizations, &requested_languages);

        CT {
            loader: loader,
            text: "".to_string(),
            picked_path: "".to_string(),
            status_text: "".to_string(),
            cursor1: 0,
            cursor2: 0,
            password: "".to_string(),
            search: "".to_string(),
            st: "".to_string(),
            r: 0..0,
            hide_password: false,
            search_bar: false,
            show_popup: false,
            popup_position: Pos2 { x: 0.0, y: 0.0 },
            panel_central: true,
            panel_setting: false,
            selected_language: "English".to_string(),
            languages: vec![
                "German".to_string(),
                "English".to_string(),
                "Thai".to_string(),
            ],
        }
    }
}

impl eframe::App for CT {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.panel_central == false && self.panel_setting == true {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(20.0);
                ui.heading("Select a Language");

                ui.add_space(10.0); // Add some spacing

                // The ComboBox widget
                ComboBox::new("my_combobox", "Select an option") // Unique ID and label
                    .selected_text(&self.selected_language) // Display the currently selected text
                    .show_ui(ui, |ui| {
                        // Iterate over your options and create a selectable item for each
                        for i in &self.languages {
                            if ui.selectable_label(false, i).clicked() {
                                self.selected_language = i.clone();
                            }
                        }
                    });

                ui.add_space(20.0); // More spacing

                ui.label(format!("You selected: {}", self.selected_language));

                ui.label("Malayalam: ഹലോ ലോകം!"); // ml-IN
                ui.label("Punjabi: ਸਤਿ ਸ੍ਰੀ ਅਕਾਲ ਦੁਨਿਆ!"); // pa-IN
                ui.label("Sinhala: ආයුබෝවන් ලෝකය!"); // si-LK

                ui.label("Tamil: வணக்கம் உலகம்!"); // ta-IN
                ui.label("Telugu: నమస్కారం ప్రపంచం!"); // te-IN

                ui.label("Yiddish: שלום עולם!"); // yi
                ui.label("Yoruba: Bawo ni aye!"); // yo-NG
                ui.label("Zulu: Sawubona Mhlaba!"); // zu-ZA

                ui.label("Chinese: 你好，世界！"); // zh-CN (covers zh-TW, zh-HK, zh-SG, zh-MO)
                ui.label("English: Hello World!"); // en-US
                ui.label("Thai: สวัสดีชาวโลก!"); // th-TH
                ui.label("Amharic: ሰላም አለም!"); // am-ET
                ui.label("Arabic: مرحبًا بالعالم!"); // ar-SA
                ui.label("Armenian: Բարև աշխարհ!"); // hy-AM
                ui.label("Azerbaijani: Salam Dünya!"); // az-AZ
                ui.label("Basque: Kaixo Mundua!"); // eu-ES
                ui.label("Belarusian: Прывітанне Сусвет!"); // be-BY

                ui.label("Bosnian: Zdravo svijete!"); // bs-BA
                ui.label("Bulgarian: Здравей свят!"); // bg-BG
                ui.label("Catalan: Hola Món!"); // ca-ES
                ui.label("Chinese: 你好，世界！"); // zh-CN (covers zh-TW, zh-HK, zh-SG, zh-MO)
                ui.label("Croatian: Zdravo svijete!"); // hr-HR
                ui.label("Czech: Ahoj světe!"); // cs-CZ
                ui.label("Danish: Hej Verden!"); // da-DK
                ui.label("Dutch: Hallo Wereld!"); // nl-NL
                ui.label("Greek: Γεια σου κόσμε!"); // el-GR

                ui.label("Esperanto: Saluton Mondo!"); // eo
                ui.label("Estonian: Tere maailm!"); // et-EE
                ui.label("Filipino: Kamusta Mundo!"); // tl-PH
                ui.label("Finnish: Hei maailma!"); // fi-FI
                ui.label("French: Bonjour le monde !"); // fr-FR
                ui.label("Frisian: Goeie dei wrâld!"); // fy-NL
                ui.label("Galician: Ola Mundo!"); // gl-ES

                ui.label("German: Hallo Welt!"); // de-DE

                ui.label("Haitian Creole: Bonjou mond!"); // ht-HT
                ui.label("Hausa: Sannu Duniya!"); // ha-NG
                ui.label("Hawaiian: Aloha honua!"); // haw-US
                ui.label("Hindi: नमस्ते दुनिया!"); // hi-IN
                ui.label("Hungarian: Helló világ!"); // hu-HU
                ui.label("Igbo: Ndewo Ụwa!"); // ig-NG
                ui.label("Irish: Dia duit an domhan!"); // ga-IE
                ui.label("Italian: Ciao mondo!"); // it-IT
                ui.label("Japanese: こんにちは世界！"); // ja-JP
                ui.label("Korean: 안녕하세요 세계!"); // ko-KR
                ui.label("Kurdish (Kurmanji): Silav cîhan!"); // ku-TR
                ui.label("Kyrgyz: Салам дүйнө!"); // ky-KG
                ui.label("Latin: Salve Mundus!"); // la
                ui.label("Latvian: Sveika pasaule!"); // lv-LV
                ui.label("Lithuanian: Labas pasauli!"); // lt-LT
                ui.label("Nepali: नमस्कार संसार!"); // ne-NP
                ui.label("Pashto: سلام نړی!"); // ps-AF
                ui.label("Persian: سلام دنیا!"); // fa-IR
                ui.label("Bengali: ওহে বিশ্ব!"); // bn-BD
                ui.label("Georgian: გამარჯობა სამყარო!"); // ka-GE
                ui.label("Gujarati: નમસ્કાર વિશ્વ!"); // gu-IN
                ui.label("Kannada: ನಮಸ್ಕಾರ ಜಗತ್ತು!"); // kn-IN
                ui.label("Khmer: សួស្តី​ពិភពលោក!"); // km-KH
                ui.label("Lao: ສະບາຍດີໂລກ!"); // lo-LA
                ui.label("Myanmar (Burmese): မင်္ဂလာပါကမ္ဘာလောက!"); // my-MM
                ui.label("Marathi: नमस्कार जग!"); // mr-IN
                ui.label("Mongolian: Сайн уу дэлхий!"); // mn-MN                
                ui.label("Kazakh: Сәлем Әлем!"); // kk-KZ                

                ui.label("Luxembourgish: Moien Welt!"); // lb-LU
                ui.label("Macedonian: Здраво свету!"); // mk-MK
                ui.label("Malagasy: Salama izao tontolo izao!"); // mg-MG
                ui.label("Malay: Hai dunia!"); // ms-MY
                ui.label("Maltese: Bongu dinja!"); // mt-MT
                ui.label("Maori: Kia ora e te ao!"); // mi-N
                ui.label("Norwegian: Hei verden!"); // no-NO
                ui.label("Polish: Witaj świecie!"); // pl-PL
                ui.label("Portuguese: Olá Mundo!"); // pt-PT
                ui.label("Romanian: Salut Lume!"); // ro-RO
                ui.label("Russian: Привет мир!"); // ru-RU
                ui.label("Samoan: Talofa le lalolagi!"); // sm-WS
                ui.label("Scottish Gaelic: Halò a shaoghal!"); // gd-GB
                ui.label("Serbian: Здраво свете!"); // sr-RS
                ui.label("Sesotho: Lumela Lefatše!"); // st-ZA
                ui.label("Shona: Mhoro!"); // sn-ZW
                ui.label("Sindhi: سلام دنيا!"); // sd-PK
                ui.label("Slovak: Ahoj svet!"); // sk-SK
                ui.label("Slovenian: Pozdravljen svet!"); // sl-SI
                ui.label("Somali: Salaam dunia!"); // so-SO
                ui.label("Spanish: ¡Hola Mundo!"); // es-ES
                ui.label("Sundanese: Sampurasun!"); // su-ID
                ui.label("Swahili: Habari dunia!"); // sw-TZ
                ui.label("Swedish: Hej Världen!"); // sv-SE
                ui.label("Tajik: Салом дунё!"); // tg-TJ
                ui.label("Turkish: Merhaba Dünya!"); // tr-TR
                ui.label("Ukrainian: Привіт Світ!"); // uk-UA
                ui.label("Urdu: السلام علیکم دنیا!"); // ur-PK
                ui.label("Uzbek: Salom dunyo!"); // uz-UZ
                ui.label("Vietnamese: Xin chào thế giới!"); // vi-VN
                ui.label("Welsh: Helo Byd!"); // cy-GB
                ui.label("Xhosa: Molo Lizwe!"); // xh-ZA                
            });
        } else if self.panel_central == true && self.panel_setting == false {
            egui::CentralPanel::default().show(ctx, |ui| {
                let r = get_char_range(self.cursor1, self.cursor2);
                let stl = self.text.char_range(r.clone()).to_string();
                //println!("{:?}", r.);

                if stl.len() > 0 {
                    self.st = stl;
                    self.r = r;
                }

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
                            layout_job.append(
                                &string[pos..pos + target_word.len()],
                                0.0,
                                color_format,
                            );

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
                        .hint_text(fl!(self.loader, "open"))
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
        };
        if self.show_popup {
            let popup_id = egui::Id::new("my_popup");
            egui::Area::new(popup_id)
                .fixed_pos(self.popup_position)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = self.st.to_string());
                            self.show_popup = false;
                        }

                        if ui.button("Paste").clicked() {
                            let txt = cli_clipboard::get_contents().unwrap();
                            let r = get_char_range(self.cursor1, self.cursor2);
                            self.text.insert_text(&txt, r.start);
                            self.show_popup = false;
                        }
                        if ui.button("Cut").clicked() {
                            ui.output_mut(|o| o.copied_text = self.st.to_string());
                            self.text.delete_char_range(self.r.clone());
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
                    if ui.button("Open").clicked() {
                        self.panel_central = true;
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.panel_central = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Copy").clicked() {
                        self.panel_central = true;
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        self.panel_central = true;
                        ui.close_menu();
                    }
                    if ui.button("Cut").clicked() {
                        self.panel_central = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Languages").clicked() {
                        self.panel_setting = true;
                        self.panel_central = false;
                        ui.close_menu();
                    }
                });
                ui.menu_button("About", |ui| {
                    if ui.button("Help").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("About CT").clicked() {
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

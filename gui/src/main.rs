use eframe::egui;
use fast_ereader::parser::epub::EpubParser;
use fast_ereader::parser::Parser;
use scraper::Html;
use fast_ereader::library::{Library, LibraryEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Fast Speed Reader - Native"),
        ..Default::default()
    };
    eframe::run_native(
        "Fast Speed Reader",
        options,
        Box::new(|_cc| Box::new(ReaderApp::default())),
    )
}

#[derive(PartialEq)]
enum AppView {
    Overview,
    Reader,
}

#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    dark_mode: bool,
    font_size: f32,
    wpm: usize,
    bionic_reading: bool,
    peripheral_chars: usize,
    focus_color: [u8; 3],
    #[serde(default)]
    font_path: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark_mode: true, // Default to dark mode if matching theme-dark
            font_size: 18.0,
            wpm: 300,
            bionic_reading: false,
            peripheral_chars: 20,
            focus_color: [255, 100, 100], // Soft red
            font_path: None,
        }
    }
}

impl Settings {
    fn get_settings_file_path() -> PathBuf {
        Library::get_app_dir().join("settings.json")
    }

    fn load() -> Self {
        let path = Self::get_settings_file_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = Self::get_settings_file_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct ProgressState {
    pub book_progress: HashMap<String, usize>,
}

impl ProgressState {
    fn get_progress_file_path() -> PathBuf {
        Library::get_app_dir().join("progress.json")
    }

    fn load() -> Self {
        let path = Self::get_progress_file_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(prog) = serde_json::from_str(&content) {
                    return prog;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = Self::get_progress_file_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }
}

struct SpeedReaderState {
    active: bool,
    words: Vec<String>,
    chapter_boundaries: Vec<usize>,
    current_index: usize,
    playing: bool,
    last_update: Option<std::time::Instant>,
}

impl Default for SpeedReaderState {
    fn default() -> Self {
        Self {
            active: false,
            words: Vec::new(),
            chapter_boundaries: Vec::new(),
            current_index: 0,
            playing: false,
            last_update: None,
        }
    }
}

struct ReaderApp {
    current_view: AppView,
    library: Library,
    progress: ProgressState,
    current_book_id: Option<String>,

    book_title: Option<String>,
    full_text: String,
    settings: Settings,
    speed_reader: SpeedReaderState,
    show_settings: bool,
    current_font_path: Option<Option<String>>,
}

impl Default for ReaderApp {
    fn default() -> Self {
        Self {
            current_view: AppView::Overview,
            library: Library::load(),
            progress: ProgressState::load(),
            current_book_id: None,

            book_title: None,
            full_text: "No book loaded. Click 'Import EPUB' to select an EPUB.".to_owned(),
            settings: Settings::load(),
            speed_reader: SpeedReaderState::default(),
            show_settings: false,
            current_font_path: None,
        }
    }
}

impl ReaderApp {
    fn load_epub(&mut self, path: &std::path::Path) {
        match EpubParser::parse_book(path) {
            Ok(book) => {
                let book_id = book.id.clone();
                self.book_title = Some(book.title.clone());
                self.current_book_id = Some(book_id.clone());
                
                // Ensure it's in library
                if !self.library.entries.contains_key(&book_id) {
                    self.library.entries.insert(book_id.clone(), LibraryEntry {
                        book: book.clone(),
                        file_path: path.to_path_buf(),
                    });
                    let _ = self.library.save();
                }

                let mut all_text = String::new();
                let mut words_list = Vec::new();
                let mut boundaries = Vec::new();
                
                if let Ok(parser) = EpubParser::new(path) {
                    for chapter in &book.chapters {
                        boundaries.push(words_list.len());
                        if let Ok(html) = parser.extract_chapter_html(&chapter.id) {
                            let document = Html::parse_document(&html);
                            // Extract text cleanly by taking text nodes and joining with space
                            let chapter_text = document.root_element().text().collect::<Vec<_>>().join(" ");
                            
                            all_text.push_str(&chapter_text);
                            all_text.push_str("\n\n");
                            
                            // Split into words
                            for word in chapter_text.split_whitespace() {
                                if !word.is_empty() {
                                    words_list.push(word.to_string());
                                }
                            }
                        }
                    }
                }
                
                self.full_text = all_text;
                self.speed_reader.words = words_list;
                self.speed_reader.chapter_boundaries = boundaries;
                
                // Restore progress
                let saved_index = self.progress.book_progress.get(&book_id).copied().unwrap_or(0);
                self.speed_reader.current_index = saved_index.min(self.speed_reader.words.len().saturating_sub(1));
                
                self.current_view = AppView::Reader;
            }
            Err(e) => {
                self.full_text = format!("Failed to parse book: {:?}", e);
                self.current_view = AppView::Reader;
            }
        }
    }

    fn save_progress(&mut self) {
        if let Some(book_id) = &self.current_book_id {
            self.progress.book_progress.insert(book_id.clone(), self.speed_reader.current_index);
            self.progress.save();
        }
    }

    fn render_bionic_word(ui: &mut egui::Ui, word: &str, font_size: f32, focus_color: egui::Color32) {
        let char_count = word.chars().count();
        if char_count == 0 {
            return;
        }
        let mid = (char_count + 1) / 2;
        
        let mut iter = word.chars();
        let bold_part = iter.by_ref().take(mid).collect::<String>();
        let normal_part = iter.collect::<String>();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label(egui::RichText::new(bold_part).size(font_size).color(focus_color).strong());
            ui.label(egui::RichText::new(normal_part).size(font_size).color(focus_color));
        });
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply font if changed
        if self.current_font_path.as_ref() != Some(&self.settings.font_path) {
            let mut fonts = egui::FontDefinitions::default();
            if let Some(path) = &self.settings.font_path {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "custom".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "custom".to_owned());
                }
            }
            ctx.set_fonts(fonts);
            self.current_font_path = Some(self.settings.font_path.clone());
        }

        // Apply theme
        if self.settings.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Handle playback timer
        if self.speed_reader.playing {
            let now = std::time::Instant::now();
            let ms_per_word = 60000 / self.settings.wpm.max(1);
            if let Some(last) = self.speed_reader.last_update {
                if now.duration_since(last).as_millis() as usize >= ms_per_word {
                    self.speed_reader.current_index += 1;
                    self.speed_reader.last_update = Some(now);
                    
                    if self.speed_reader.current_index % 50 == 0 {
                        self.save_progress();
                    }

                    if self.speed_reader.current_index >= self.speed_reader.words.len() {
                        self.speed_reader.playing = false;
                        self.speed_reader.current_index = self.speed_reader.words.len().saturating_sub(1);
                        self.save_progress();
                    }
                }
            } else {
                self.speed_reader.last_update = Some(now);
            }
            ctx.request_repaint(); // Important: keep repainting to progress the timer!
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.current_view == AppView::Reader {
                    if ui.button("⬅ Library").clicked() {
                        self.speed_reader.playing = false;
                        self.save_progress();
                        self.current_view = AppView::Overview;
                    }
                    if let Some(title) = &self.book_title {
                        ui.label(egui::RichText::new(title).strong());
                    }
                } else {
                    ui.label(egui::RichText::new("Library Overview").strong());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                    if self.current_view == AppView::Reader {
                        if ui.button("⚡ Speed Read").clicked() {
                            self.speed_reader.active = !self.speed_reader.active;
                            if !self.speed_reader.active {
                                self.speed_reader.playing = false;
                                self.save_progress();
                            }
                        }
                    }
                });
            });
        });

        if self.show_settings {
            egui::Window::new("Settings").open(&mut self.show_settings).show(ctx, |ui| {
                let mut changed = false;
                changed |= ui.checkbox(&mut self.settings.dark_mode, "Dark Mode").changed();
                changed |= ui.add(egui::Slider::new(&mut self.settings.font_size, 12.0..=48.0).text("Font Size")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.settings.peripheral_chars, 0..=60).text("Peripheral Chars")).changed();
                changed |= ui.checkbox(&mut self.settings.bionic_reading, "Bionic Reading").changed();
                ui.horizontal(|ui| {
                    ui.label("Focus Color");
                    changed |= ui.color_edit_button_srgb(&mut self.settings.focus_color).changed();
                });
                
                ui.horizontal(|ui| {
                    ui.label("Custom Font:");
                    let font_label = match &self.settings.font_path {
                        Some(p) => std::path::Path::new(p)
                            .file_name()
                            .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                            .to_string_lossy()
                            .to_string(),
                        None => "Default".to_string(),
                    };
                    if ui.button(font_label).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Fonts", &["ttf", "otf"])
                            .pick_file()
                        {
                            self.settings.font_path = Some(path.to_string_lossy().to_string());
                            changed = true;
                        }
                    }
                    if self.settings.font_path.is_some() {
                        if ui.button("❌").clicked() {
                            self.settings.font_path = None;
                            changed = true;
                        }
                    }
                });
                
                if changed {
                    self.settings.save();
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.current_view == AppView::Overview {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("➕ Import EPUB").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("EPUB", &["epub"])
                            .pick_file()
                        {
                            self.load_epub(&path);
                        }
                    }
                });
                ui.add_space(20.0);
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut to_remove = None;
                    let mut to_read = None;
                    for (id, entry) in &self.library.entries {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&entry.book.title).size(20.0).strong());
                                    if let Some(author) = &entry.book.author {
                                        ui.label(format!("By {}", author));
                                    }
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑 Delete").clicked() {
                                        to_remove = Some(id.clone());
                                    }
                                    if ui.button("📖 Read").clicked() {
                                        to_read = Some(entry.file_path.clone());
                                    }
                                });
                            });
                        });
                        ui.add_space(10.0);
                    }
                    if let Some(id) = to_remove {
                        self.library.entries.remove(&id);
                        let _ = self.library.save();
                        self.progress.book_progress.remove(&id);
                        self.progress.save();
                    }
                    if let Some(path) = to_read {
                        self.load_epub(&path);
                    }
                });
            } else if self.speed_reader.active {
                // Speed Reader Mode
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    
                    // Controls
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        
                        if ui.button("⏮ Chapter").clicked() {
                            let curr = self.speed_reader.current_index;
                            let mut prev = 0;
                            for &bound in self.speed_reader.chapter_boundaries.iter().rev() {
                                if bound < curr {
                                    prev = bound;
                                    break;
                                }
                            }
                            self.speed_reader.current_index = prev;
                            self.save_progress();
                        }
                        if ui.button("⏪ 10").clicked() {
                            self.speed_reader.current_index = self.speed_reader.current_index.saturating_sub(10);
                            self.save_progress();
                        }
                        let play_label = if self.speed_reader.playing { "Pause" } else { "Play" };
                        if ui.button(play_label).clicked() {
                            self.speed_reader.playing = !self.speed_reader.playing;
                            self.speed_reader.last_update = None;
                            if self.speed_reader.current_index >= self.speed_reader.words.len() {
                                self.speed_reader.current_index = 0;
                            }
                            if !self.speed_reader.playing {
                                self.save_progress();
                            }
                        }
                        if ui.button("⏩ 10").clicked() {
                            self.speed_reader.current_index = (self.speed_reader.current_index + 10).min(self.speed_reader.words.len().saturating_sub(1));
                            self.save_progress();
                        }
                        if ui.button("⏭ Chapter").clicked() {
                            let curr = self.speed_reader.current_index;
                            let mut next = self.speed_reader.words.len().saturating_sub(1);
                            for &bound in &self.speed_reader.chapter_boundaries {
                                if bound > curr {
                                    next = bound;
                                    break;
                                }
                            }
                            self.speed_reader.current_index = next;
                            self.save_progress();
                        }
                    });
                    
                    ui.add_space(10.0);
                    if ui.add(egui::Slider::new(&mut self.settings.wpm, 100..=1000).text("WPM")).changed() {
                        self.settings.save();
                    }
                    
                    ui.add_space(50.0);
                    
                    // Word Display
                    if self.speed_reader.words.is_empty() {
                        ui.label(egui::RichText::new("No words loaded.").size(self.settings.font_size));
                    } else {
                        let current_word = &self.speed_reader.words[self.speed_reader.current_index];
                        
                        // Calculate left peripheral words
                        let mut left_words = Vec::new();
                        let mut left_chars = 0;
                        for i in (0..self.speed_reader.current_index).rev() {
                            let w = &self.speed_reader.words[i];
                            let len = w.chars().count() + 1; // +1 for space
                            if left_chars + len > self.settings.peripheral_chars {
                                break;
                            }
                            left_words.insert(0, w.clone());
                            left_chars += len;
                        }
                        
                        // Calculate right peripheral words
                        let mut right_words = Vec::new();
                        let mut right_chars = 0;
                        for i in (self.speed_reader.current_index + 1)..self.speed_reader.words.len() {
                            let w = &self.speed_reader.words[i];
                            let len = w.chars().count() + 1; // +1 for space
                            if right_chars + len > self.settings.peripheral_chars {
                                break;
                            }
                            right_words.push(w.clone());
                            right_chars += len;
                        }
                        
                        let left_text = left_words.join(" ");
                        let right_text = right_words.join(" ");
                        
                        let font_id = egui::FontId::proportional(self.settings.font_size);
                        let left_str = if left_text.is_empty() { String::new() } else { format!("{} ", left_text) };
                        let right_str = if right_text.is_empty() { String::new() } else { format!(" {}", right_text) };
                        
                        let left_width = ui.fonts(|f| f.layout_no_wrap(left_str.clone(), font_id.clone(), egui::Color32::WHITE).rect.width());
                        let center_width = ui.fonts(|f| f.layout_no_wrap(current_word.to_string(), font_id.clone(), egui::Color32::WHITE).rect.width());
                        
                        let panel_width = ui.available_width();
                        let exact_offset = (panel_width / 2.0) - left_width - (center_width / 2.0);
                        
                        let focus_c = egui::Color32::from_rgb(
                            self.settings.focus_color[0],
                            self.settings.focus_color[1],
                            self.settings.focus_color[2],
                        );

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.add_space(exact_offset.max(0.0));
                            
                            // Left peripheral
                            if !left_str.is_empty() {
                                ui.label(egui::RichText::new(left_str)
                                    .size(self.settings.font_size)
                                    .color(egui::Color32::from_gray(128)));
                            }
                            
                            // Center word
                            if self.settings.bionic_reading {
                                Self::render_bionic_word(ui, current_word, self.settings.font_size, focus_c);
                            } else {
                                ui.label(egui::RichText::new(current_word)
                                    .size(self.settings.font_size)
                                    .color(focus_c)
                                    .strong());
                            }
                            
                            // Right peripheral
                            if !right_str.is_empty() {
                                ui.label(egui::RichText::new(right_str)
                                    .size(self.settings.font_size)
                                    .color(egui::Color32::from_gray(128)));
                            }
                            
                            // Fill remaining width to ensure the block spans the entire panel,
                            // preventing `vertical_centered` from shifting it.
                            ui.add_space(ui.available_width());
                        });
                    }
                    
                    ui.add_space(50.0);
                    
                    // Progress Slider
                    let mut progress = self.speed_reader.current_index;
                    let max_progress = self.speed_reader.words.len().saturating_sub(1);
                    if max_progress > 0 {
                        if ui.add(egui::Slider::new(&mut progress, 0..=max_progress).show_value(false)).changed() {
                            self.speed_reader.current_index = progress;
                            // avoid saving on every micro-drag to not kill disk
                        }
                        if ui.input(|i| i.pointer.any_released()) {
                            self.save_progress(); // save when user drops the slider
                        }
                        
                        let pct = (progress as f64 / max_progress as f64 * 100.0).floor();
                        ui.label(format!("{} / {} ({}%)", progress + 1, max_progress + 1, pct));
                    }
                });
            } else {
                // Standard Read Mode
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(egui::RichText::new(&self.full_text).size(self.settings.font_size));
                });
            }
        });
    }
}

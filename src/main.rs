#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use xor_112::i18n::tr;
mod editor;
mod icons;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(tr("XOR-112"))
            .with_icon(icons::app_icon())
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1000.0, 650.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        tr("XOR-112"),
        options,
        Box::new(|cc| Ok(Box::new(editor::Editor::new(cc)))),
    )
}

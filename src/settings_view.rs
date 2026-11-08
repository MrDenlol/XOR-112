use super::*;
impl Editor {
    pub(super) fn preferences(&self) -> Preferences {
        Preferences {
            language: self.language,
            theme: self.theme,
            grid: self.grid,
            snap: self.snap,
            frames: self.show_frames,
            animation: self.animate_wires,
            zoom: self.zoom,
        }
    }
    pub(super) fn restore_preferences(&mut self, ctx: &egui::Context, path: Option<PathBuf>) {
        self.settings_path = path;
        if let Some(path) = &self.settings_path {
            match xor_112::preferences::load_with_legacy(path) {
                Ok(p) => {
                    self.language = p.language;
                    self.theme = p.theme;
                    self.grid = p.grid;
                    self.snap = p.snap;
                    self.show_frames = p.frames;
                    self.animate_wires = p.animation;
                    self.zoom = p.zoom;
                    self.fit_next = false;
                    self.settings_seen = p.clone();
                    self.settings_saved = p;
                }
                Err(_) => self.message(
                    tr("Не удалось прочитать настройки. Используются стандартные параметры."),
                    true,
                ),
            }
        }
        xor_112::i18n::set_language(self.language);
        appearance::apply(ctx, self.theme);
    }
    pub(super) fn prepare_preferences(&mut self, ctx: &egui::Context) {
        xor_112::i18n::set_language(self.language);
        appearance::set_theme(self.theme);
        if ctx.style().visuals.selection.stroke.color != accent() {
            appearance::apply(ctx, self.theme);
        }
    }
    pub(super) fn persist_preferences(&mut self, ctx: &egui::Context) {
        let p = self.preferences();
        if p != self.settings_seen {
            self.settings_seen = p;
            self.save_preferences_at =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(400));
            ctx.request_repaint();
        }
        if let Some(at) = self.save_preferences_at {
            if std::time::Instant::now() >= at || ctx.input(|i| i.viewport().close_requested()) {
                self.flush_preferences();
            } else {
                ctx.request_repaint_after(at.saturating_duration_since(std::time::Instant::now()));
            }
        }
    }
    pub(super) fn flush_preferences(&mut self) {
        self.save_preferences_at = None;
        let p = self.preferences();
        if p == self.settings_saved {
            return;
        }
        if let Some(path) = &self.settings_path {
            match xor_112::preferences::save(path, &p) {
                Ok(()) => self.settings_saved = p,
                Err(_) => self.message(
                    tr("Не удалось сохранить настройки. Проверьте доступ к папке настроек."),
                    true,
                ),
            }
        }
    }
}

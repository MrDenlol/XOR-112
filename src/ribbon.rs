use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tab {
    Home,
    Settings,
}
impl Editor {
    pub(super) fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(tint(Color32::from_rgb(232, 242, 235)))
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(26., 26.), Sense::hover());
                    icons::logo(ui.painter(), r);
                    ui.add_space(10.);
                    for (tab, label, width) in [
                        (Tab::Home, tr("Главная"), 88.),
                        (Tab::Settings, tr("Настройки"), 112.),
                    ] {
                        let active = self.tab == tab;
                        let r = ui.add_sized(
                            [width, 32.],
                            egui::Button::new(
                                RichText::new(label)
                                    .size(14.)
                                    .color(if active { accent() } else { ink() })
                                    .strong(),
                            )
                            .frame(false)
                            .corner_radius(4),
                        );
                        if r.clicked() {
                            self.tab = tab;
                        }
                        if active {
                            ui.painter().line_segment(
                                [
                                    r.rect.left_bottom() + vec2(8., 0.),
                                    r.rect.right_bottom() - vec2(8., 0.),
                                ],
                                Stroke::new(2.5_f32, accent()),
                            );
                        }
                    }
                    if ui
                        .add_sized([98., 32.], egui::Button::new(tr("Справка")).frame(false))
                        .on_hover_text(tr("Сочетания клавиш · F1"))
                        .clicked()
                    {
                        self.help_open = true;
                    }
                });
                egui::Frame::new()
                    .fill(panel())
                    .corner_radius(9)
                    .inner_margin(egui::Margin::symmetric(10, 5))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::ScrollArea::horizontal()
                            .id_salt("ribbon_scroll")
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = vec2(3., 3.);
                                ui.style_mut()
                                    .text_styles
                                    .insert(egui::TextStyle::Body, FontId::proportional(13.));
                                ui.horizontal_top(|ui| match self.tab {
                                    Tab::Home => {
                                        group(ui, 154., tr("Файл"), |ui| {
                                            ui.add_space(10.);
                                            self.file_buttons(ui);
                                        });
                                        divider(ui);
                                        group(ui, 154., tr("Редактирование"), |ui| {
                                            ui.add_space(10.);
                                            ui.horizontal(|ui| {
                                                if icons::button(
                                                    ui,
                                                    Icon::Undo,
                                                    tr("Отменить · Ctrl+Z"),
                                                    !self.undo.is_empty(),
                                                )
                                                .clicked()
                                                {
                                                    self.undo();
                                                }
                                                if icons::button(
                                                    ui,
                                                    Icon::Redo,
                                                    tr("Повторить · Ctrl+Y"),
                                                    !self.redo.is_empty(),
                                                )
                                                .clicked()
                                                {
                                                    self.redo();
                                                }
                                                if icons::button(
                                                    ui,
                                                    Icon::Copy,
                                                    tr("Дублировать · Ctrl+D"),
                                                    self.selected.is_some(),
                                                )
                                                .clicked()
                                                {
                                                    self.duplicate();
                                                }
                                                if icons::button(
                                                    ui,
                                                    Icon::Delete,
                                                    tr("Удалить · Delete"),
                                                    self.selected.is_some()
                                                        || self.selected_wire.is_some(),
                                                )
                                                .clicked()
                                                {
                                                    self.delete();
                                                }
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 60., tr("Проверка"), |ui| {
                                            ui.add_space(10.);
                                            ui.horizontal(|ui| {
                                                ui.add_space(12.);
                                                if icons::button(
                                                    ui,
                                                    Icon::Table,
                                                    tr("Таблица истинности"),
                                                    true,
                                                )
                                                .clicked()
                                                {
                                                    self.truth = Some(self.circuit.truth_table());
                                                }
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 332., tr("Отображение"), |ui| {
                                            ui.horizontal_top(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.set_width(155.);
                                                    ui.checkbox(&mut self.grid, tr("Сетка"));
                                                    ui.add_enabled_ui(
                                                        self.circuit.alu64.is_none(),
                                                        |ui| {
                                                            ui.checkbox(
                                                                &mut self.snap,
                                                                tr("Привязка к сетке"),
                                                            );
                                                            ui.checkbox(
                                                                &mut self.show_frames,
                                                                tr("Рамки элементов"),
                                                            );
                                                        },
                                                    );
                                                });
                                                ui.vertical(|ui| {
                                                    ui.checkbox(
                                                        &mut self.animate_wires,
                                                        tr("Анимация проводов"),
                                                    );
                                                });
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 196., tr("Масштаб"), |ui| {
                                            ui.add_space(10.);
                                            ui.allocate_ui_with_layout(
                                                vec2(196., 38.),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    if icons::button(
                                                        ui,
                                                        Icon::Minus,
                                                        tr("Уменьшить масштаб"),
                                                        true,
                                                    )
                                                    .clicked()
                                                    {
                                                        self.set_zoom(self.zoom / 1.15);
                                                    }
                                                    self.zoom_control(ui, "ribbon_zoom");
                                                    if icons::button(
                                                        ui,
                                                        Icon::Plus,
                                                        tr("Увеличить масштаб"),
                                                        true,
                                                    )
                                                    .clicked()
                                                    {
                                                        self.set_zoom(self.zoom * 1.15);
                                                    }
                                                    if icons::button(
                                                        ui,
                                                        Icon::Fit,
                                                        tr("Показать всю схему · F"),
                                                        true,
                                                    )
                                                    .clicked()
                                                    {
                                                        self.fit_next = true;
                                                    }
                                                },
                                            );
                                        });
                                    }
                                    Tab::Settings => {
                                        group(ui, 240., tr("Язык"), |ui| {
                                            ui.add_space(12.);
                                            ui.horizontal(|ui| {
                                                for (lang, name) in [
                                                    (Language::Russian, tr("Русский")),
                                                    (Language::English, "English"),
                                                ] {
                                                    if ui
                                                        .add(
                                                            egui::Button::new(name)
                                                                .selected(self.language == lang)
                                                                .corner_radius(6),
                                                        )
                                                        .clicked()
                                                    {
                                                        self.language = lang;
                                                        xor_112::i18n::set_language(lang);
                                                        ctx.request_repaint();
                                                    }
                                                }
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 355., tr("Тема"), |ui| {
                                            ui.add_space(12.);
                                            ui.horizontal(|ui| {
                                                for (theme, name) in [
                                                    (Theme::Green, tr("Зелёная")),
                                                    (Theme::Blue, tr("Синяя")),
                                                    (Theme::Gray, tr("Серая")),
                                                ] {
                                                    if ui
                                                        .add(
                                                            egui::Button::new(name)
                                                                .selected(self.theme == theme)
                                                                .corner_radius(6),
                                                        )
                                                        .clicked()
                                                    {
                                                        self.theme = theme;
                                                        appearance::apply(ctx, theme);
                                                        ctx.request_repaint();
                                                    }
                                                }
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 280., tr("Параметры"), |ui| {
                                            ui.add_space(12.);
                                            ui.label(tr("Настройки сохраняются автоматически"));
                                        });
                                    }
                                });
                            });
                    });
            });
    }
    fn file_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if icons::button(ui, Icon::New, tr("Новая схема · Ctrl+N"), true).clicked() {
                self.request(Action::New);
            }
            if icons::button(ui, Icon::Open, tr("Загрузить схему · Ctrl+O"), true).clicked()
            {
                self.request(Action::Open);
            }
            if icons::button(ui, Icon::Save, tr("Сохранить · Ctrl+S"), true).clicked() {
                self.save(false);
            }
            let more = icons::button(ui, Icon::More, tr("Ещё: сохранить как и примеры"), true);
            let popup = ui.make_persistent_id("file_more");
            if more.clicked() {
                ui.memory_mut(|m| m.toggle_popup(popup));
            }
            egui::popup_below_widget(
                ui,
                popup,
                &more,
                egui::PopupCloseBehavior::CloseOnClick,
                |ui| {
                    ui.set_min_width(235.);
                    if ui.button(tr("Сохранить как…")).clicked() {
                        ui.memory_mut(|m| m.close_popup());
                        self.save(true);
                    }
                    ui.separator();
                    for (label, action) in [
                        (tr("Пример: полусумматор"), Action::Half),
                        (tr("Пример: полный сумматор"), Action::Full),
                        (tr("Пример: АЛУ · 32 бита"), Action::Alu32),
                        (tr("Пример: АЛУ · 64 бита"), Action::Alu64),
                    ] {
                        if ui.button(label).clicked() {
                            ui.memory_mut(|m| m.close_popup());
                            self.request(action);
                        }
                    }
                },
            );
        });
    }
}
fn group(ui: &mut egui::Ui, width: f32, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| {
        ui.set_width(width);
        ui.allocate_ui_with_layout(
            vec2(width, 64.),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_min_height(64.0);
                body(ui);
            },
        );
        ribbon_caption(ui, title, width);
    });
}
fn divider(ui: &mut egui::Ui) {
    let (r, _) = ui.allocate_exact_size(vec2(8., 84.), Sense::hover());
    ui.painter().line_segment(
        [r.center_top(), r.center_bottom()],
        Stroke::new(1.0_f32, border()),
    );
}

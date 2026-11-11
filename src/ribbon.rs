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
                    .fill(workspace())
                    .inner_margin(egui::Margin::symmetric(10, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(28., 28.), Sense::hover());
                    icons::logo(ui.painter(), rect, accent());
                    ui.add_space(6.);
                    ui.label(RichText::new("XOR-112").size(20.).strong());
                    ui.add_space(12.);
                    ui.separator();
                    ui.add_space(12.);
                    ui.label(
                        RichText::new(tr("Редактор логических схем"))
                            .size(13.)
                            .color(muted()),
                    );
                });
                ui.add_space(7.);
                ui.horizontal(|ui| {
                    for (tab, icon, label, width) in [
                        (Tab::Home, Icon::Home, tr("Главная"), 132.),
                        (Tab::Settings, Icon::Settings, tr("Настройки"), 150.),
                    ] {
                        if icons::tab(ui, icon, label, self.tab == tab, width).clicked() {
                            self.tab = tab;
                        }
                    }
                    if icons::tab(ui, Icon::Help, tr("Справка"), false, 132.)
                        .on_hover_text(tr("Сочетания клавиш · F1"))
                        .clicked()
                    {
                        self.help_open = true;
                    }
                });
                ui.add_space(4.);
                egui::Frame::new()
                    .fill(panel())
                    .corner_radius(13)
                    .stroke(Stroke::new(1.0_f32, border()))
                    .shadow(soft_shadow())
                    .inner_margin(egui::Margin::symmetric(12, 8))
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
                                        group(ui, 222., tr("Файл"), |ui| {
                                            ui.add_space(2.);
                                            self.file_buttons(ui);
                                        });
                                        divider(ui);
                                        group(ui, 246., tr("Редактирование"), |ui| {
                                            ui.add_space(2.);
                                            ui.horizontal(|ui| {
                                                if icons::command(
                                                    ui,
                                                    Icon::Undo,
                                                    tr("Отменить"),
                                                    tr("Отменить · Ctrl+Z"),
                                                    !self.undo.is_empty(),
                                                    58.,
                                                )
                                                .clicked()
                                                {
                                                    self.undo();
                                                }
                                                if icons::command(
                                                    ui,
                                                    Icon::Redo,
                                                    tr("Вернуть"),
                                                    tr("Повторить · Ctrl+Y"),
                                                    !self.redo.is_empty(),
                                                    58.,
                                                )
                                                .clicked()
                                                {
                                                    self.redo();
                                                }
                                                if icons::command(
                                                    ui,
                                                    Icon::Copy,
                                                    tr("Копировать"),
                                                    tr("Дублировать · Ctrl+D"),
                                                    self.selected.is_some(),
                                                    58.,
                                                )
                                                .clicked()
                                                {
                                                    self.duplicate();
                                                }
                                                if icons::command(
                                                    ui,
                                                    Icon::Delete,
                                                    tr("Удалить"),
                                                    tr("Удалить · Delete"),
                                                    self.selected.is_some()
                                                        || self.selected_wire.is_some(),
                                                    58.,
                                                )
                                                .clicked()
                                                {
                                                    self.delete();
                                                }
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 84., tr("Проверка"), |ui| {
                                            ui.add_space(2.);
                                            ui.horizontal(|ui| {
                                                ui.add_space(13.);
                                                if icons::command(
                                                    ui,
                                                    Icon::Table,
                                                    tr("Таблица"),
                                                    tr("Таблица истинности"),
                                                    true,
                                                    58.,
                                                )
                                                .clicked()
                                                {
                                                    self.truth = Some(self.circuit.truth_table());
                                                }
                                            });
                                        });
                                        divider(ui);
                                        group(ui, 366., tr("Отображение"), |ui| {
                                            ui.horizontal_top(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.set_width(136.);
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
                                                    ui.checkbox(
                                                        &mut self.hide_properties,
                                                        tr("Скрыть свойства"),
                                                    );
                                                    ui.checkbox(
                                                        &mut self.hide_library,
                                                        tr("Скрыть библиотеку элементов"),
                                                    );
                                                });
                                            });
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
            if icons::command(
                ui,
                Icon::New,
                tr("Создать"),
                tr("Новая схема · Ctrl+N"),
                true,
                58.,
            )
            .clicked()
            {
                self.request(Action::New);
            }
            if icons::command(
                ui,
                Icon::Open,
                tr("Открыть"),
                tr("Загрузить схему · Ctrl+O"),
                true,
                58.,
            )
            .clicked()
            {
                self.request(Action::Open);
            }
            if icons::command(
                ui,
                Icon::Save,
                tr("Сохранить"),
                tr("Сохранить · Ctrl+S"),
                true,
                64.,
            )
            .clicked()
            {
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
                        (
                            tr("Пример: большинство от трёх переменных"),
                            Action::Majority,
                        ),
                        (tr("Пример: мультиплексор на 8 входов"), Action::Mux8),
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

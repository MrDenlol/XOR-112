use super::*;

impl Editor {
    pub(super) fn help_dialog(&mut self, ctx: &egui::Context) {
        if !self.help_open {
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.help_open = false;
            return;
        }
        egui::Modal::new(Id::new("keyboard_help")).show(ctx,|ui| {
            ui.set_width(590.0_f32.min(ctx.screen_rect().width()-64.0));
            ui.heading(tr("Справка · управление"));
            ui.add_space(4.0);
            egui::ScrollArea::vertical().max_height((ctx.screen_rect().height()-180.0).max(180.0)).show(ui,|ui| {
                help_table(ui,tr("Файлы и редактирование"),&[
                    ("Ctrl + N",tr("Новая схема")),("Ctrl + O",tr("Загрузить схему")),
                    ("Ctrl + S",tr("Сохранить")),("Ctrl + Shift + S",tr("Сохранить как…")),
                    ("Ctrl + Z",tr("Отменить действие")),("Ctrl + Y / Ctrl + Shift + Z",tr("Повторить действие")),
                    ("Ctrl + D",tr("Дублировать выбранный элемент без проводов")),
                    ("Delete / Backspace",tr("Удалить выбранный элемент или провод")),
                    (tr("Пробел"),tr("Переключить выбранный вход между 0 и 1")),
                ]);
                help_table(ui,tr("Холст и окна"),&[
                    ("F",tr("Показать всю схему")),("F1",tr("Открыть / закрыть справку")),
                    ("Esc",tr("Отменить размещение или соединение; закрыть диалог")),
                    ("Enter",tr("Применить имя элемента или введённый масштаб")),
                    (tr("Esc в поле масштаба"),tr("Отменить изменение масштаба")),
                ]);
                help_table(ui,tr("Мышь"),&[
                    (tr("Колесо над холстом"),tr("Изменить масштаб относительно указателя")),
                    (tr("Перетянуть фон"),tr("Переместить вид схемы")),
                    (tr("Средняя / правая кнопка + движение"),tr("Переместить вид схемы")),
                    (tr("Перетянуть элемент"),tr("Переместить элемент")),
                    (tr("Нажатие на название"),tr("Переименовать элемент")),
                    (tr("Нажатие на вход"),tr("Переключить бит")),
                    (tr("Выходной контакт, затем входной"),tr("Соединить элементы; можно в обратном порядке")),
                    (tr("Правая кнопка на проводе / входном контакте"),tr("Удалить соединение")),
                    (tr("Нажатие на проценты"),tr("Ввести масштаб от 0,5 до 250%")),
                    (tr("Готовые схемы"),tr("Выберите схему, затем нажмите на холст для вставки")),
                    (tr("Нажатие на элемент общего плана АЛУ"),tr("Приблизить элемент для редактирования")),
                ]);
                help_table(ui,tr("Текстовые поля"),&[
                    ("Ctrl + A",tr("Выделить весь текст")),("Ctrl + C / X / V",tr("Копировать / вырезать / вставить текст")),
                    ("Ctrl + Z / Ctrl + Shift + Z",tr("Отменить / повторить ввод текста")),
                    (tr("Стрелки, Home / End"),tr("Перемещать курсор")),(tr("Shift + клавиши перемещения"),tr("Выделять текст")),
                    (tr("Ctrl + Влево / Вправо"),tr("Перемещаться по словам")),("Ctrl + Backspace / Delete",tr("Удалить слово")),
                    ("Tab / Shift + Tab",tr("Перейти к следующему / предыдущему полю или кнопке")),
                ]);
                ui.add_space(8.0);
                ui.label(tr("В текстовых полях клавиши редактируют текст, а не схему. На macOS вместо Ctrl используется Command; перемещение по словам — Option."));
                ui.add_space(8.0);
                ui.label(tr("Отображение: сетка, привязка, рамки и анимация проводов настраиваются только в верхней ленте. Белый свет на активном проводе обозначает сигнал 1 и движется к приёмнику. Анимация не меняет результаты расчёта."));
            });
            ui.add_space(8.0);
            if ui.add_sized([110.0,32.0],egui::Button::new(tr("Закрыть"))).clicked(){self.help_open=false;}
        });
    }
}
fn help_table(ui: &mut egui::Ui, title: &str, rows: &[(&str, &str)]) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).strong().color(accent()));
    ui.separator();
    egui::Grid::new(title)
        .striped(true)
        .num_columns(2)
        .min_col_width(260.0)
        .max_col_width(280.0)
        .spacing(vec2(18.0, 8.0))
        .show(ui, |ui| {
            for (key, action) in rows {
                ui.add(egui::Label::new(RichText::new(*key).size(12.0)).wrap());
                ui.add(egui::Label::new(RichText::new(*action).size(12.0)).wrap());
                ui.end_row();
            }
        });
}

use super::*;

pub(super) fn panel_heading(ui: &mut egui::Ui, icon: Icon, title: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(19., 24.), Sense::hover());
        icons::paint(
            ui.painter(),
            Rect::from_center_size(r.center(), vec2(18., 18.)),
            icon,
            accent(),
        );
        ui.label(RichText::new(title).size(13.).strong());
    });
    ui.add_space(3.);
    ui.separator();
    ui.add_space(4.);
}

pub(super) fn result_band(ui: &mut egui::Ui, value: Option<Bit>) {
    egui::Frame::new()
        .fill(tint(Color32::from_rgb(231, 246, 235)))
        .corner_radius(9)
        .inner_margin(10.)
        .show(ui, |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.), Sense::hover());
            icons::paint(
                ui.painter(),
                Rect::from_center_size(r.left_center() + vec2(11., 0.), vec2(21., 21.)),
                Icon::Pulse,
                accent(),
            );
            ui.painter().text(
                r.left_center() + vec2(32., 0.),
                Align2::LEFT_CENTER,
                tr("Результат"),
                FontId::proportional(14.),
                ink(),
            );
            ui.painter().text(
                r.right_center(),
                Align2::RIGHT_CENTER,
                bit_text(value),
                FontId::proportional(27.),
                signal_color(value),
            );
        });
}

fn palette_button(ui: &mut egui::Ui, symbol: &str, label: &str, active: bool) -> egui::Response {
    let (r, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, label)
    });
    let fill = if active {
        ui.visuals().selection.bg_fill
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        tint(Color32::from_rgb(246, 248, 247))
    };
    ui.painter().rect(
        r,
        8.,
        fill,
        Stroke::new(
            if active { 1.4_f32 } else { 1.0_f32 },
            if active { accent() } else { border() },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        r.left_center() + vec2(23., 0.),
        Align2::CENTER_CENTER,
        symbol,
        FontId::proportional(18.),
        ink(),
    );
    ui.painter().text(
        r.left_center() + vec2(48., 0.),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.),
        ink(),
    );
    response
}

impl Editor {
    pub(super) fn palette(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("library")
            .default_width(228.)
            .width_range(0.0..=228.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(card_frame())
            .show(ctx, |ui| {
                ui.set_min_width(0.);
                panel_heading(ui, Icon::Library, tr("Библиотека элементов"));
                egui::ScrollArea::vertical()
                    .id_salt("library_scroll")
                    .show(ui, |ui| {
                        self.netlist_navigation(ui);
                        egui::CollapsingHeader::new(RichText::new(tr("Элементы")).strong())
                            .id_salt("basic_elements")
                            .default_open(true)
                            .show_unindented(ui, |ui| {
                                if palette_button(
                                    ui,
                                    "↖",
                                    tr("Выбор"),
                                    self.tool.is_none() && self.template.is_none(),
                                )
                                .clicked()
                                {
                                    self.tool = None;
                                    self.template = None;
                                    self.pending_pin = None;
                                }
                                for kind in [Kind::Input, Kind::Output] {
                                    self.library_item(ui, kind);
                                }
                            });
                        ui.add_space(8.);
                        egui::CollapsingHeader::new(
                            RichText::new(tr("Логические вентили")).strong(),
                        )
                        .id_salt("logic_elements")
                        .default_open(true)
                        .show_unindented(ui, |ui| {
                            for kind in Kind::ALL.into_iter().skip(2) {
                                self.library_item(ui, kind);
                            }
                        });
                        ui.add_space(8.);
                        egui::CollapsingHeader::new(RichText::new(tr("Готовые схемы")).strong())
                            .id_salt("circuit_templates")
                            .default_open(true)
                            .show_unindented(ui, |ui| {
                                for template in Template::ALL {
                                    // Text-only buttons give long translated names the full width.
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 36.],
                                            egui::Button::new(template.name())
                                                .selected(self.template == Some(template))
                                                .corner_radius(8),
                                        )
                                        .on_hover_text(tr(
                                            "Выберите схему, затем нажмите на холст для вставки",
                                        ))
                                        .clicked()
                                    {
                                        self.tool = None;
                                        self.pending_pin = None;
                                        self.template = Some(template);
                                        self.message(
                                            tr("Нажмите на холст для вставки схемы"),
                                            false,
                                        );
                                    }
                                }
                            });
                    });
            });
    }
    fn library_item(&mut self, ui: &mut egui::Ui, kind: Kind) {
        let symbol = match kind {
            Kind::Input | Kind::Output => "○",
            Kind::And | Kind::Nand => "&",
            Kind::Or | Kind::Nor => "≥1",
            Kind::Not => "¬",
            Kind::Xor => "!=",
            Kind::Xnor => "=",
            Kind::Buffer => ">",
        };
        if palette_button(ui, symbol, kind.name(), self.tool == Some(kind))
            .on_hover_text(kind.description())
            .clicked()
        {
            self.tool = Some(kind);
            self.template = None;
            self.pending_pin = None;
            self.message(
                if xor_112::i18n::is_english() {
                    format!("Selected element: {}", kind.name())
                } else {
                    format!("Выбран элемент: {}", kind.name())
                },
                false,
            );
        }
    }
    pub(super) fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(workspace())
                    .inner_margin(egui::Margin::symmetric(18, 5)),
            )
            .show(ctx, |ui| {
                let (row, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 38.), Sense::hover());
                let controls = Rect::from_min_max(pos2(row.right() - 204., row.top()), row.max);
                let message = Rect::from_min_max(
                    row.min,
                    pos2(
                        (row.left() + 280.).min(controls.left() - 300.),
                        row.bottom(),
                    ),
                );
                let circle = message.left_center() + vec2(10., 0.);
                ui.painter().circle_filled(
                    circle,
                    9.,
                    if self.status_error { error() } else { accent() },
                );
                if self.status_error {
                    ui.painter().text(
                        circle,
                        Align2::CENTER_CENTER,
                        "!",
                        FontId::proportional(14.),
                        Color32::WHITE,
                    );
                } else {
                    icons::paint(
                        ui.painter(),
                        Rect::from_center_size(circle, vec2(14., 14.)),
                        Icon::Check,
                        Color32::WHITE,
                    );
                }
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_max(message.min + vec2(28., 0.), message.max))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(xor_112::i18n::display_message(&self.status))
                                    .size(12.)
                                    .color(if self.status_error { error() } else { ink() }),
                            )
                            .truncate(),
                        );
                    },
                );
                let counts = Rect::from_min_max(
                    pos2(message.right() + 14., row.top()),
                    pos2(controls.left() - 10., row.bottom()),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(counts)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.separator();
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                tr("Элементов:"),
                                self.circuit.nodes.len()
                            ))
                            .size(12.)
                            .color(muted()),
                        );
                        ui.add_space(8.);
                        ui.separator();
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                tr("Соединений:"),
                                self.circuit.wires.len()
                            ))
                            .size(12.)
                            .color(muted()),
                        );
                    },
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(controls)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.;
                        if ui
                            .add_sized([32., 32.], egui::Button::new("−").corner_radius(8))
                            .on_hover_text(tr("Уменьшить масштаб"))
                            .clicked()
                        {
                            self.set_zoom(self.zoom / 1.15);
                        }
                        self.zoom_control(ui, "status_zoom");
                        if ui
                            .add_sized([32., 32.], egui::Button::new("+").corner_radius(8))
                            .on_hover_text(tr("Увеличить масштаб"))
                            .clicked()
                        {
                            self.set_zoom(self.zoom * 1.15);
                        }
                        if icons::button(ui, Icon::Fit, tr("Показать всю схему · F"), true)
                            .clicked()
                        {
                            self.fit_next = true;
                        }
                    },
                );
            });
    }
}

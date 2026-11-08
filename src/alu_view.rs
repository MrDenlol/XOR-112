use super::*;
use xor_112::alu64::{binary, grouped, parse_word, Operation};

impl Editor {
    pub(super) fn sync_alu_inputs(&mut self) {
        if let Some(alu) = &self.circuit.alu64 {
            self.alu_a = grouped(alu.a.0);
            self.alu_b = grouped(alu.b.0);
        }
    }
    fn choose_operation(&mut self, operation: Operation) {
        if self
            .circuit
            .alu64
            .as_ref()
            .is_some_and(|a| a.operation != operation)
        {
            self.checkpoint();
            self.circuit.alu64.as_mut().unwrap().operation = operation;
            self.message(
                if xor_112::i18n::is_english() {
                    format!("{} · code {}", operation.description(), operation.code())
                } else {
                    format!("{} · код {}", operation.description(), operation.code())
                },
                false,
            );
        }
    }
    pub(super) fn alu_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("alu_controls").exact_width(265.0).resizable(false)
            .frame(egui::Frame::new().fill(panel()).inner_margin(16.0)).show(ctx,|ui|{
            egui::ScrollArea::vertical().show(ui,|ui|{
                ui.heading(tr("АЛУ · 64 бита"));
                ui.label(RichText::new(tr("Входы без знака. Старший бит слева.")).size(12.0).color(muted()));
                ui.label(RichText::new("A").strong());
                ui.add(egui::TextEdit::multiline(&mut self.alu_a).id(Id::new("alu_a")).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).desired_rows(4).char_limit(95));
                ui.label(RichText::new("B").strong());
                ui.add(egui::TextEdit::multiline(&mut self.alu_b).id(Id::new("alu_b")).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).desired_rows(4).char_limit(95));
                let values=parse_word(&self.alu_a).zip(parse_word(&self.alu_b));
                let changed=values.is_some_and(|(a,b)|{let alu=self.circuit.alu64.as_ref().unwrap();a!=alu.a||b!=alu.b});
                if values.is_none(){ui.colored_label(error(),tr("Введите от 1 до 64 бит: только 0 и 1. Пробелы допустимы."));}
                if ui.add_enabled(changed,egui::Button::new(tr("Применить входы"))).clicked(){
                    let (a,b)=values.unwrap();self.checkpoint();let alu=self.circuit.alu64.as_mut().unwrap();alu.a=a;alu.b=b;
                    self.sync_alu_inputs();self.message(tr("Входы АЛУ обновлены"),false);
                }
                if changed{ui.label(RichText::new(tr("Новые биты ещё не применены")).size(11.0).color(error()));}
                ui.separator();ui.label(RichText::new(tr("ОПЕРАЦИЯ")).size(11.0).color(muted()));
                for op in Operation::ALL{
                    let selected=self.circuit.alu64.as_ref().unwrap().operation==op;
                    if ui.add_sized([225.0,29.0],egui::Button::new(format!("{}     {}",op.code(),op.title())).selected(selected).corner_radius(9)).on_hover_text(op.description()).clicked(){self.choose_operation(op);}
                }
                let alu=self.circuit.alu64.as_ref().unwrap();let result=alu.evaluate();
                ui.separator();
                if result.divide_by_zero{ui.colored_label(error(),tr("Деление на ноль. Результат и остаток не определены."));}
                else{
                    word_readout(ui,tr("R · младшие 64 бита"),result.low);
                    if ui.small_button(tr("Скопировать R")).clicked(){ctx.copy_text(binary(result.low));}
                    if matches!(alu.operation,Operation::Multiply|Operation::Add){word_readout(ui,tr("HI · старшие 64 бита"),result.high);}
                    if matches!(alu.operation,Operation::Divide|Operation::Sqrt){word_readout(ui,tr("Остаток"),result.remainder);}
                }
                ui.add_space(8.0);ui.label(RichText::new(tr("ФЛАГИ")).size(11.0).color(muted()));
                for (name,value) in [(tr("Перенос"),result.carry),(tr("Заём"),result.borrow),(tr("Переполнение"),result.overflow),(tr("Деление на ноль"),result.divide_by_zero)]{
                    ui.horizontal(|ui|{ui.label(RichText::new(if value{"1"}else{"0"}).monospace().strong().color(if value{high()}else{low()}));ui.label(name);});
                }
                ui.add_space(12.0);
                ui.label(RichText::new(tr("При переполнении R содержит младшие биты. Вычитание выполняется по модулю 2⁶⁴. Корень целый; B при извлечении корня не используется.")).size(12.0).color(muted()));
                ui.label(RichText::new(tr("Это фиксированная функциональная схема. Входы и операция сохраняются в файле; развёртки блоков в вентили нет.")).size(11.0).color(muted()));
            });
        });
    }
    pub(super) fn alu_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(paper()))
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
                self.canvas_rect = rect;
                if self.fit_next {
                    self.zoom = ((rect.width() - 24.0) / 1000.0)
                        .min((rect.height() - 90.0) / 580.0)
                        .clamp(0.25, 1.2);
                    self.pan =
                        (rect.size() - vec2(1000.0, 580.0) * self.zoom) * 0.5 + vec2(0.0, 30.0);
                    self.fit_next = false;
                }
                let blocked = self.confirm.is_some() || self.truth.is_some() || self.help_open;
                if !blocked {
                    if response.dragged() {
                        self.pan += ctx.input(|i| i.pointer.delta());
                    }
                    if let Some(p) = response.hover_pos() {
                        let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                        if scroll != 0.0 {
                            let before = self.world(p, rect);
                            self.zoom = (self.zoom * (scroll * 0.002).exp()).clamp(0.25, 2.5);
                            self.pan = p - rect.min - before.to_vec2() * self.zoom;
                        }
                    }
                }
                let areas: Vec<_> = Operation::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(i, op)| {
                        (
                            op,
                            Rect::from_min_size(
                                self.screen(pos2(330.0, 50.0 + i as f32 * 105.0), rect),
                                vec2(200.0, 76.0) * self.zoom,
                            ),
                        )
                    })
                    .collect();
                let a_rect = Rect::from_min_size(
                    self.screen(pos2(30.0, 125.0), rect),
                    vec2(215.0, 130.0) * self.zoom,
                );
                let b_rect = Rect::from_min_size(
                    self.screen(pos2(30.0, 340.0), rect),
                    vec2(215.0, 130.0) * self.zoom,
                );
                if !blocked && response.clicked() {
                    if let Some(p) = response.interact_pointer_pos() {
                        if let Some((op, _)) = areas.iter().find(|(_, r)| r.contains(p)) {
                            self.choose_operation(*op);
                        } else if a_rect.contains(p) {
                            ctx.memory_mut(|m| m.request_focus(Id::new("alu_a")));
                        } else if b_rect.contains(p) {
                            ctx.memory_mut(|m| m.request_focus(Id::new("alu_b")));
                        }
                    }
                }
                let alu = self.circuit.alu64.as_ref().unwrap();
                let result = alu.evaluate();
                let z = self.zoom;
                let painter = ui.painter_at(rect);
                if self.grid {
                    let step = 20.0 * z;
                    let origin = rect.min + self.pan;
                    for vertical in [true, false] {
                        let (low, high, offset) = if vertical {
                            (rect.left(), rect.right(), origin.x)
                        } else {
                            (rect.top(), rect.bottom(), origin.y)
                        };
                        for index in ((low - offset) / step).floor() as i32
                            ..=((high - offset) / step).ceil() as i32
                        {
                            let p = offset + index as f32 * step;
                            let points = if vertical {
                                [pos2(p, rect.top()), pos2(p, rect.bottom())]
                            } else {
                                [pos2(rect.left(), p), pos2(rect.right(), p)]
                            };
                            painter.line_segment(
                                points,
                                Stroke::new(0.5_f32, tint(Color32::from_rgb(228, 237, 231))),
                            );
                        }
                    }
                }
                let mux = Rect::from_min_size(
                    self.screen(pos2(620.0, 160.0), rect),
                    vec2(85.0, 300.0) * z,
                );
                let output = Rect::from_min_size(
                    self.screen(pos2(765.0, 215.0), rect),
                    vec2(215.0, 180.0) * z,
                );
                for (i, (op, r)) in areas.iter().enumerate() {
                    let color = if *op == alu.operation {
                        accent()
                    } else {
                        tint(Color32::from_rgb(195, 209, 200))
                    };
                    let a = pos2(r.left(), r.top() + 24.0 * z);
                    wire_animation::draw_signal_wire(
                        &painter,
                        a_rect.right_center(),
                        a,
                        Stroke::new(1.8 * z, color),
                        self.animate_wires && *op == alu.operation,
                        ctx.input(|i| i.time),
                        i as f64,
                    );
                    if *op != Operation::Sqrt {
                        wire_animation::draw_signal_wire(
                            &painter,
                            b_rect.right_center(),
                            pos2(r.left(), r.top() + 52.0 * z),
                            Stroke::new(1.8 * z, color),
                            self.animate_wires && *op == alu.operation,
                            ctx.input(|i| i.time),
                            i as f64 + 10.,
                        );
                    }
                    let port = pos2(mux.left(), mux.top() + (25.0 + i as f32 * 60.0) * z);
                    wire_animation::draw_signal_wire(
                        &painter,
                        r.right_center(),
                        port,
                        Stroke::new(2.0 * z, color),
                        self.animate_wires && *op == alu.operation,
                        ctx.input(|i| i.time),
                        i as f64 + 20.,
                    );
                }
                wire_animation::draw_signal_wire(
                    &painter,
                    mux.right_center(),
                    output.left_center(),
                    Stroke::new(
                        2.8 * z,
                        if result.divide_by_zero {
                            error()
                        } else {
                            accent()
                        },
                    ),
                    self.animate_wires && !result.divide_by_zero,
                    ctx.input(|i| i.time),
                    30.,
                );
                source_card(&painter, a_rect, "A", alu.a.0, z);
                source_card(&painter, b_rect, "B", alu.b.0, z);
                for (op, r) in &areas {
                    let selected = *op == alu.operation;
                    painter.rect(
                        *r,
                        10.0,
                        if selected {
                            tint(Color32::from_rgb(224, 242, 231))
                        } else {
                            panel()
                        },
                        Stroke::new(
                            if selected { 2.0_f32 } else { 1.0_f32 },
                            if selected { accent() } else { border() },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        r.center() - vec2(0.0, 9.0 * z),
                        Align2::CENTER_CENTER,
                        op.title(),
                        FontId::proportional(23.0 * z),
                        if selected { accent() } else { ink() },
                    );
                    painter.text(
                        r.center() + vec2(0.0, 21.0 * z),
                        Align2::CENTER_CENTER,
                        format!("{} · {}", op.code(), op.description()),
                        FontId::proportional(10.0 * z),
                        muted(),
                    );
                }
                painter.rect(
                    mux,
                    12.0,
                    panel(),
                    Stroke::new(1.5 * z, accent()),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    mux.center() - vec2(0.0, 20.0 * z),
                    Align2::CENTER_CENTER,
                    "MUX",
                    FontId::proportional(18.0 * z),
                    accent(),
                );
                painter.text(
                    mux.center() + vec2(0.0, 12.0 * z),
                    Align2::CENTER_CENTER,
                    alu.operation.code(),
                    FontId::monospace(18.0 * z),
                    ink(),
                );
                painter.rect(
                    output,
                    12.0,
                    panel(),
                    Stroke::new(
                        2.0 * z,
                        if result.divide_by_zero {
                            error()
                        } else {
                            accent()
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    output.center_top() + vec2(0.0, 22.0 * z),
                    Align2::CENTER_CENTER,
                    tr("R · 64 бита"),
                    FontId::proportional(18.0 * z),
                    ink(),
                );
                painter.text(
                    output.center(),
                    Align2::CENTER_CENTER,
                    if result.divide_by_zero {
                        "—".into()
                    } else {
                        grouped(result.low)
                    },
                    FontId::monospace(13.0 * z),
                    if result.divide_by_zero {
                        error()
                    } else {
                        accent()
                    },
                );
                painter.text(
                    output.center_bottom() - vec2(0.0, 20.0 * z),
                    Align2::CENTER_CENTER,
                    if result.divide_by_zero {
                        tr("Ошибка: деление на ноль")
                    } else {
                        tr("Результат выбранной операции")
                    },
                    FontId::proportional(10.0 * z),
                    muted(),
                );
            });
    }
}
fn source_card(p: &egui::Painter, r: Rect, title: &str, value: u64, z: f32) {
    p.rect(
        r,
        12.0,
        panel(),
        Stroke::new(1.5 * z, accent()),
        egui::StrokeKind::Inside,
    );
    p.text(
        r.center_top() + vec2(0.0, 20.0 * z),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0 * z),
        accent(),
    );
    p.text(
        r.center() + vec2(0.0, 13.0 * z),
        Align2::CENTER_CENTER,
        grouped(value),
        FontId::monospace(13.0 * z),
        ink(),
    );
}
fn word_readout(ui: &mut egui::Ui, label: &str, value: u64) {
    ui.label(RichText::new(label).size(12.0).color(muted()));
    ui.label(
        RichText::new(grouped(value))
            .monospace()
            .size(14.0)
            .color(accent()),
    );
}

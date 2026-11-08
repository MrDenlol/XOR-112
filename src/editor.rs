use crate::icons::{self, Icon};
use xor_112::i18n::tr;
#[path = "alu_view.rs"]
mod alu_view;
#[path = "appearance.rs"]
mod appearance;
#[path = "help.rs"]
mod help;
#[path = "netlist_view.rs"]
mod netlist_view;
#[path = "ribbon.rs"]
mod ribbon;
#[path = "settings_view.rs"]
mod settings_view;
#[path = "wire_animation.rs"]
mod wire_animation;
use appearance::*;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Id, Key, PointerButton, Pos2, Rect, RichText, Sense,
    Stroke, Vec2,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};
use xor_112::preferences::{Language, Preferences, Theme};
use xor_112::{
    model::{Bit, Circuit, Evaluation, Kind, Node, NodeId, TruthTable},
    storage,
};

const NODE_SIZE: Vec2 = vec2(120.0, 80.0);
const GRID_STEP: f32 = 20.0;

#[derive(Clone, Copy, PartialEq)]
enum Pin {
    Out(NodeId),
    In(NodeId, usize),
}
#[derive(Clone, Copy)]
enum Action {
    New,
    Open,
    Half,
    Full,
    Alu32,
    Alu64,
    Quit,
}

struct Rename {
    id: NodeId,
    draft: String,
    focus: bool,
}

pub struct Editor {
    circuit: Circuit,
    saved: Circuit,
    path: Option<PathBuf>,
    selected: Option<NodeId>,
    selected_wire: Option<usize>,
    tool: Option<Kind>,
    pending_pin: Option<Pin>,
    undo: Vec<Circuit>,
    redo: Vec<Circuit>,
    zoom: f32,
    pan: Vec2,
    drag_node: Option<NodeId>,
    drag_origin: Option<(Pos2, Pos2)>,
    canvas_rect: Rect,
    fit_next: bool,
    grid: bool,
    snap: bool,
    show_frames: bool,
    animate_wires: bool,
    help_open: bool,
    status: String,
    status_error: bool,
    confirm: Option<Action>,
    allow_close: bool,

    truth: Option<Result<TruthTable, String>>,
    zoom_edit: Option<(Id, String, bool)>,
    alu_a: String,
    alu_b: String,
    rename: Option<Rename>,
    node_index: HashMap<NodeId, usize>,
    sources: HashMap<(NodeId, usize), NodeId>,
    evaluation: Option<(u64, Arc<Evaluation>)>,
    find_gate: String,
    language: Language,
    theme: Theme,
    tab: ribbon::Tab,
    settings_path: Option<PathBuf>,
    settings_seen: Preferences,
    settings_saved: Preferences,
    save_preferences_at: Option<std::time::Instant>,
}

impl Editor {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::from_context(&cc.egui_ctx);
        app.restore_preferences(&cc.egui_ctx, xor_112::preferences::path());
        app.circuit = Circuit::half_adder();
        app.saved = app.circuit.clone();
        if let Some(path) = std::env::args_os().nth(1) {
            let path = PathBuf::from(path);
            match storage::load(&path) {
                Ok(c) => app.replace(c, Some(path)),
                Err(e) => app.message(e, true),
            }
        }
        app
    }
    pub fn from_context(ctx: &egui::Context) -> Self {
        xor_112::i18n::set_language(Language::Russian);
        appearance::set_theme(Theme::Green);
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.override_text_color = Some(ink());
        style.visuals.panel_fill = panel();
        style.visuals.window_fill = panel();
        style.visuals.extreme_bg_color = Color32::WHITE;
        style.visuals.faint_bg_color = paper();
        style.visuals.selection.bg_fill = tint(Color32::from_rgb(216, 237, 223));
        style.visuals.selection.stroke = Stroke::new(1.0_f32, accent());
        style.visuals.widgets.inactive.bg_fill = tint(Color32::from_rgb(239, 245, 241));
        style.visuals.widgets.inactive.weak_bg_fill = tint(Color32::from_rgb(239, 245, 241));
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border());
        style.visuals.widgets.hovered.bg_fill = tint(Color32::from_rgb(221, 239, 227));
        for widget in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = egui::CornerRadius::same(9);
        }
        style.visuals.window_corner_radius = egui::CornerRadius::same(14);
        style.visuals.menu_corner_radius = egui::CornerRadius::same(10);
        style.spacing.item_spacing = vec2(10.0, 10.0);
        style.spacing.button_padding = vec2(12.0, 8.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(21.0));
        // Store the custom style for the explicit theme: the first native theme
        // event must not switch back to egui's default square controls.
        ctx.set_theme(egui::ThemePreference::Light);
        ctx.set_style_of(egui::Theme::Light, style);
        let circuit = Circuit::half_adder();
        Self {
            saved: circuit.clone(),
            circuit,
            path: None,
            selected: None,
            selected_wire: None,
            tool: None,
            pending_pin: None,
            undo: vec![],
            redo: vec![],
            zoom: 1.0,
            pan: vec2(40.0, 70.0),
            drag_node: None,
            drag_origin: None,
            canvas_rect: Rect::NOTHING,
            fit_next: true,
            grid: true,
            snap: true,
            show_frames: true,
            animate_wires: true,
            help_open: false,
            status: tr("Готово").into(),
            status_error: false,
            confirm: None,
            allow_close: false,

            truth: None,
            zoom_edit: None,
            alu_a: String::new(),
            alu_b: String::new(),
            rename: None,
            node_index: HashMap::new(),
            sources: HashMap::new(),
            evaluation: None,
            find_gate: String::new(),
            language: Language::Russian,
            theme: Theme::Green,
            tab: ribbon::Tab::Home,
            settings_path: None,
            settings_seen: Preferences::default(),
            settings_saved: Preferences::default(),
            save_preferences_at: None,
        }
    }
    fn dirty(&self) -> bool {
        self.circuit != self.saved
    }
    fn message(&mut self, s: impl Into<String>, error: bool) {
        self.status = s.into();
        self.status_error = error;
    }
    fn checkpoint(&mut self) {
        if self.undo.last() != Some(&self.circuit) {
            self.undo.push(self.circuit.clone());
        }
        if self.undo.len()
            > if self.circuit.nodes.len() > 10000 {
                6
            } else {
                80
            }
        {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
    fn undo(&mut self) {
        if let Some(c) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.circuit, c));
            self.clear_selection();
            self.message(tr("Действие отменено"), false);
        }
    }
    fn redo(&mut self) {
        if let Some(c) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.circuit, c));
            self.clear_selection();
            self.message(tr("Действие повторено"), false);
        }
    }
    fn clear_selection(&mut self) {
        self.sync_alu_inputs();
        self.sync_netlist_inputs();
        self.selected = None;
        self.selected_wire = None;
        self.pending_pin = None;
        self.drag_node = None;
        self.drag_origin = None;
        self.rename = None;
    }
    fn replace(&mut self, c: Circuit, path: Option<PathBuf>) {
        self.saved = c.clone();
        self.circuit = c;
        self.path = path;
        self.undo.clear();
        self.redo.clear();
        self.clear_selection();
        self.tool = None;
        self.fit_next = true;
        self.truth = None;
        self.message(tr("Схема открыта"), false);
    }
    fn save(&mut self, save_as: bool) -> bool {
        let target = if save_as || self.path.is_none() {
            let name = self
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("scheme.binary");
            rfd::FileDialog::new()
                .set_title(tr("Сохранить схему"))
                .add_filter(tr("Бинарная схема"), &["binary"])
                .set_file_name(name)
                .save_file()
        } else {
            self.path.clone()
        };
        let Some(mut path) = target else {
            return false;
        };
        if path.extension().is_none() {
            path.set_extension("binary");
        }
        match storage::save(&path, &self.circuit) {
            Ok(()) => {
                self.saved = self.circuit.clone();
                self.message(
                    if xor_112::i18n::is_english() {
                        format!("Saved: {}", path.display())
                    } else {
                        format!("Сохранено: {}", path.display())
                    },
                    false,
                );
                self.path = Some(path);
                true
            }
            Err(e) => {
                self.message(e, true);
                false
            }
        }
    }
    fn request(&mut self, action: Action) {
        if self.dirty() {
            self.confirm = Some(action);
        } else {
            self.perform(action);
        }
    }
    fn perform(&mut self, action: Action) {
        match action {
            Action::New => self.replace(Circuit::default(), None),
            Action::Half => self.replace(Circuit::half_adder(), None),
            Action::Full => self.replace(Circuit::full_adder(), None),
            Action::Alu32 => self.replace(Circuit::alu32_example(), None),
            Action::Alu64 => self.replace(Circuit::alu64_example(), None),
            Action::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title(tr("Открыть схему"))
                    .add_filter(tr("Бинарная схема"), &["binary", "json"])
                    .pick_file()
                {
                    match storage::load(&path) {
                        Ok(c) => self.replace(c, Some(path)),
                        Err(e) => self.message(e, true),
                    }
                }
            }
            Action::Quit => self.allow_close = true,
        }
    }
    fn delete(&mut self) {
        if let Some(id) = self.selected {
            self.checkpoint();
            self.circuit.remove(id);
            self.clear_selection();
        } else if let Some(i) = self.selected_wire {
            if i < self.circuit.wires.len() {
                self.checkpoint();
                self.circuit.wires.remove(i);
                self.selected_wire = None;
            }
        }
    }
    fn duplicate(&mut self) {
        if let Some(n) = self.selected.and_then(|id| self.circuit.node(id)).cloned() {
            self.checkpoint();
            let position = snap_position(pos2(n.x + 40.0, n.y + 120.0), self.snap);
            match self.circuit.add(n.kind, position.x, position.y) {
                Ok(id) => {
                    let copy = self.circuit.nodes.iter_mut().find(|n| n.id == id).unwrap();
                    copy.label = n.label;
                    copy.value = n.value;
                    self.selected = Some(id);
                }
                Err(e) => self.message(e, true),
            }
        }
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::F1))
            && self.confirm.is_none()
            && self.rename.is_none()
            && self.truth.is_none()
        {
            self.help_open = !self.help_open;
        }
        if self.help_open {
            return;
        }
        if self.zoom_edit.is_some() {
            return;
        }
        if self.confirm.is_some() || self.truth.is_some() || self.rename.is_some() {
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.pending_pin = None;
            self.tool = None;
            self.clear_selection();
        }
        if ctx.wants_keyboard_input() {
            return;
        }
        let command = ctx.input(|i| i.modifiers.command);
        if command {
            if ctx.input(|i| i.key_pressed(Key::S)) {
                self.save(ctx.input(|i| i.modifiers.shift));
            }
            if ctx.input(|i| i.key_pressed(Key::O)) {
                self.request(Action::Open);
            }
            if ctx.input(|i| i.key_pressed(Key::N)) {
                self.request(Action::New);
            }
            if ctx.input(|i| i.key_pressed(Key::Z)) {
                if ctx.input(|i| i.modifiers.shift) {
                    self.redo();
                } else {
                    self.undo();
                }
            }
            if ctx.input(|i| i.key_pressed(Key::Y)) {
                self.redo();
            }
            if ctx.input(|i| i.key_pressed(Key::D)) {
                self.duplicate();
            }
        } else {
            if ctx.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
                self.delete();
            }
            if ctx.input(|i| i.key_pressed(Key::F)) {
                self.fit_next = true;
            }
            if ctx.input(|i| i.key_pressed(Key::Space)) {
                if let Some(n) = self
                    .selected
                    .and_then(|id| self.circuit.node(id))
                    .cloned()
                    .filter(|n| n.kind == Kind::Input)
                {
                    self.checkpoint();
                    self.circuit
                        .nodes
                        .iter_mut()
                        .find(|x| x.id == n.id)
                        .unwrap()
                        .value = n.value.flipped();
                }
            }
        }
    }
    fn zoom_control(&mut self, ui: &mut egui::Ui, key: &str) {
        let id = ui.make_persistent_id(key);
        let editing = self
            .zoom_edit
            .as_ref()
            .is_some_and(|(active, _, _)| *active == id);
        if editing {
            let (_, mut text, focus) = self.zoom_edit.take().unwrap();
            let mut output = egui::TextEdit::singleline(&mut text)
                .id(id)
                .desired_width(60.0)
                .min_size(vec2(72.0, 32.0))
                .margin(vec2(6.0, 5.0))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
                .char_limit(8)
                .hint_text("100%")
                .show(ui);
            if focus {
                output.response.request_focus();
                output
                    .state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(0),
                        egui::text::CCursor::new(text.chars().count()),
                    )));
                output.state.store(ui.ctx(), id);
            }
            let escape = ui.input(|i| i.key_pressed(Key::Escape));
            let enter = ui.input(|i| i.key_pressed(Key::Enter));
            let finished = enter || (!focus && output.response.lost_focus());
            if escape {
                output.response.surrender_focus();
            } else if finished {
                if let Some(zoom) = parse_zoom(&text) {
                    self.set_zoom(zoom);
                    output.response.surrender_focus();
                } else {
                    self.message(tr("Введите масштаб от 0,5 до 250 процентов"), true);
                    if enter {
                        self.zoom_edit = Some((id, text, true));
                    }
                }
            } else {
                self.zoom_edit = Some((id, text, false));
            }
        } else if ui
            .add_sized(
                [72.0, 32.0],
                egui::Button::new(format!("{}%", zoom_text(self.zoom))).corner_radius(9),
            )
            .on_hover_text(tr(
                "Нажмите и введите масштаб: 0,5–250%. Enter — применить, Esc — отменить.",
            ))
            .clicked()
        {
            self.zoom_edit = Some((id, zoom_text(self.zoom), true));
        }
    }
    fn set_zoom(&mut self, zoom: f32) {
        if self.canvas_rect.is_positive() {
            let center = self.canvas_rect.center();
            let world = self.world(center, self.canvas_rect);
            self.zoom = zoom.clamp(0.005, 2.5);
            self.pan = center - self.canvas_rect.min - world.to_vec2() * self.zoom;
        } else {
            self.zoom = zoom.clamp(0.005, 2.5);
        }
    }
    fn palette(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("library")
            .exact_width(204.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(panel()).inner_margin(18.0))
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(tr("ЭЛЕМЕНТЫ"))
                        .size(11.0)
                        .strong()
                        .color(muted()),
                );
                ui.add_space(6.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.netlist_navigation(ui);
                    if ui
                        .add_sized(
                            [168.0, 36.0],
                            egui::Button::new(tr("↖   Выбор"))
                                .selected(self.tool.is_none())
                                .corner_radius(9),
                        )
                        .clicked()
                    {
                        self.tool = None;
                        self.pending_pin = None;
                    }
                    ui.add_space(4.0);
                    for (i, kind) in Kind::ALL.into_iter().enumerate() {
                        if i == 2 {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(tr("ЛОГИЧЕСКИЕ ВЕНТИЛИ"))
                                    .size(10.0)
                                    .color(muted()),
                            );
                        }
                        let symbol = match kind {
                            Kind::Input => "○",
                            Kind::Output => "○",
                            Kind::And | Kind::Nand => "&",
                            Kind::Or | Kind::Nor => "≥1",
                            Kind::Not => "¬",
                            Kind::Xor => "!=",
                            Kind::Xnor => "=",
                            Kind::Buffer => ">",
                        };
                        if ui
                            .add_sized(
                                [168.0, 35.0],
                                egui::Button::new(format!("{symbol}     {}", kind.name()))
                                    .selected(self.tool == Some(kind))
                                    .corner_radius(9),
                            )
                            .on_hover_text(kind.description())
                            .clicked()
                        {
                            self.tool = Some(kind);
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
                });
            });
    }
    fn inspector(&mut self, ctx: &egui::Context, eval: &Evaluation) {
        egui::SidePanel::right("inspector")
            .exact_width(234.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(panel()).inner_margin(18.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(tr("СХЕМА"))
                            .size(11.0)
                            .strong()
                            .color(muted()),
                    );
                    ui.add_space(7.0);
                    let mut title = self.circuit.title.clone();
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut title)
                            .desired_width(194.0)
                            .char_limit(120)
                            .hint_text(tr("Название схемы")),
                    );
                    if response.changed() {
                        self.checkpoint();
                        self.circuit.title = title;
                    }
                    ui.label(
                        RichText::new(if self.dirty() {
                            tr("Есть несохранённые изменения")
                        } else if self.path.is_some() {
                            tr("Все изменения сохранены")
                        } else {
                            tr("Схема ещё не записана в файл")
                        })
                        .size(11.0)
                        .color(muted()),
                    );
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(10.0);
                    if let Some(n) = self.selected.and_then(|id| self.circuit.node(id)).cloned() {
                        ui.label(
                            RichText::new(tr("ВЫБРАННЫЙ ЭЛЕМЕНТ"))
                                .size(10.0)
                                .color(muted()),
                        );
                        ui.heading(n.kind.name());
                        ui.label(RichText::new(n.kind.description()).size(12.0));
                        ui.add_space(8.0);
                        ui.label(RichText::new(tr("Подпись")).size(12.0).color(muted()));
                        let mut label = n.label.clone();
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut label)
                                    .char_limit(48)
                                    .desired_width(194.0),
                            )
                            .changed()
                        {
                            self.checkpoint();
                            self.circuit
                                .nodes
                                .iter_mut()
                                .find(|x| x.id == n.id)
                                .unwrap()
                                .label = label;
                        }
                        ui.add_space(8.0);
                        if n.kind == Kind::Input {
                            ui.horizontal(|ui| {
                                ui.label(tr("Значение"));
                                if ui
                                    .button(
                                        RichText::new(n.value.text())
                                            .size(22.0)
                                            .color(signal_color(Some(n.value))),
                                    )
                                    .clicked()
                                {
                                    self.checkpoint();
                                    self.circuit
                                        .nodes
                                        .iter_mut()
                                        .find(|x| x.id == n.id)
                                        .unwrap()
                                        .value = n.value.flipped();
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(tr("Результат"));
                                let value = eval.values.get(&n.id).copied().flatten();
                                ui.label(
                                    RichText::new(bit_text(value))
                                        .size(24.0)
                                        .color(signal_color(value)),
                                );
                            });
                        }
                        if ui
                            .button(tr("Дублировать"))
                            .on_hover_text(tr("Ctrl+D · без соединений"))
                            .clicked()
                        {
                            self.duplicate();
                        }
                        if ui
                            .button(tr("Удалить элемент"))
                            .on_hover_text("Delete")
                            .clicked()
                        {
                            self.delete();
                        }
                    } else if self.selected_wire.is_some() {
                        ui.heading(tr("Соединение"));
                        ui.label(tr("Выбран провод."));
                        if ui.button(tr("Удалить провод")).clicked() {
                            self.delete();
                        }
                    } else {
                        ui.label(
                            RichText::new(tr("СВОЙСТВА ЭЛЕМЕНТА"))
                                .size(10.0)
                                .color(muted()),
                        );
                        ui.label(
                            RichText::new(tr("Элемент не выбран"))
                                .size(13.0)
                                .color(muted()),
                        );
                    }
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(10.0);
                    if self.circuit.netlist.is_some() {
                        self.netlist_controls(ui, eval);
                    } else {
                        ui.label(RichText::new(tr("ВЫХОДЫ СХЕМЫ")).size(10.0).color(muted()));
                        let outputs: Vec<_> = self
                            .circuit
                            .nodes
                            .iter()
                            .filter(|n| n.kind == Kind::Output)
                            .collect();
                        if outputs.is_empty() {
                            ui.label(
                                RichText::new(tr("Добавьте элемент «Выход»"))
                                    .size(12.0)
                                    .color(muted()),
                            );
                        }
                        for n in outputs {
                            let value = eval.values.get(&n.id).copied().flatten();
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(bit_text(value))
                                        .size(23.0)
                                        .strong()
                                        .color(signal_color(value)),
                                );
                                ui.add(
                                    egui::Label::new(if n.label.is_empty() {
                                        tr("Выход")
                                    } else {
                                        &n.label
                                    })
                                    .wrap(),
                                );
                            });
                        }
                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(10.0);
                    }
                    ui.label(RichText::new(tr("ПРОВЕРКА")).size(10.0).color(muted()));
                    if self.circuit.nodes.is_empty() {
                        ui.label(RichText::new(tr("Холст пуст")).color(muted()));
                    } else if eval.missing.is_empty() {
                        ui.label(
                            RichText::new(tr("Все контакты подключены"))
                                .size(12.0)
                                .color(high()),
                        );
                    } else {
                        ui.label(RichText::new(tr("Есть свободные входы")).color(error()));
                        ui.label(
                            RichText::new(tr("«—» означает, что результат пока нельзя вычислить."))
                                .size(12.0)
                                .color(muted()),
                        );
                        for &(id, port) in eval.missing.iter().take(5) {
                            if let Some(n) = self.circuit.node(id) {
                                let name = if n.label.is_empty() {
                                    n.kind.name()
                                } else {
                                    &n.label
                                };
                                if ui
                                    .small_button(if xor_112::i18n::is_english() {
                                        format!(
                                            "{} · input {}",
                                            name,
                                            if port == 0 { "A" } else { "B" }
                                        )
                                    } else {
                                        format!(
                                            "{} · вход {}",
                                            name,
                                            if port == 0 { "A" } else { "B" }
                                        )
                                    })
                                    .clicked()
                                {
                                    self.selected = Some(id);
                                    self.selected_wire = None;
                                }
                            }
                        }
                    }
                });
            });
    }
    fn screen(&self, p: Pos2, rect: Rect) -> Pos2 {
        rect.min + self.pan + p.to_vec2() * self.zoom
    }
    fn world(&self, p: Pos2, rect: Rect) -> Pos2 {
        ((p - rect.min - self.pan) / self.zoom).to_pos2()
    }
    fn node_rect(&self, n: &Node, rect: Rect) -> Rect {
        Rect::from_min_size(self.screen(pos2(n.x, n.y), rect), NODE_SIZE * self.zoom)
    }
    fn pin_pos(&self, pin: Pin, rect: Rect) -> Option<Pos2> {
        let (id, x, y) = match pin {
            Pin::Out(id) => (id, 120.0, 40.0),
            Pin::In(id, p) => {
                let n = self.indexed_node(id)?;
                (
                    id,
                    0.0,
                    if n.kind.inputs() == 1 {
                        40.0
                    } else if p == 0 {
                        20.0
                    } else {
                        60.0
                    },
                )
            }
        };
        let n = self.indexed_node(id)?;
        Some(self.screen(pos2(n.x + x, n.y + y), rect))
    }
    fn pin_at(&self, p: Pos2, rect: Rect) -> Option<Pin> {
        if self.zoom < 0.3 {
            return None;
        }
        for n in self.circuit.nodes.iter().rev() {
            if !self.node_rect(n, rect).expand(14.0).contains(p) {
                continue;
            }
            let pins = (0..n.kind.inputs())
                .map(|port| Pin::In(n.id, port))
                .chain(n.kind.has_output().then_some(Pin::Out(n.id)));
            for pin in pins {
                if self
                    .pin_pos(pin, rect)
                    .is_some_and(|v| v.distance(p) < 11.0 * self.zoom.max(0.8))
                {
                    return Some(pin);
                }
            }
        }
        None
    }
    fn fit(&mut self, rect: Rect) {
        if self.circuit.nodes.is_empty() {
            self.zoom = 1.0;
            self.pan = vec2(60.0, 100.0);
            return;
        }
        let mut bounds = Rect::NOTHING;
        for n in &self.circuit.nodes {
            bounds = bounds.union(Rect::from_min_size(pos2(n.x, n.y), NODE_SIZE));
        }
        let available = rect.size() - vec2(80.0, 80.0);
        self.zoom = (available.x / bounds.width())
            .min(available.y / bounds.height())
            .clamp(0.005, 1.25);
        self.pan = (rect.size() - bounds.size() * self.zoom) * 0.5
            - bounds.min.to_vec2() * self.zoom
            + vec2(0.0, 10.0);
    }
    fn connect_pin(&mut self, pin: Pin) {
        if let Some(start) = self.pending_pin {
            let connection = match (start, pin) {
                (Pin::Out(a), Pin::In(b, p)) | (Pin::In(b, p), Pin::Out(a)) => Some((a, b, p)),
                _ => None,
            };
            if let Some((a, b, p)) = connection {
                let old = self.circuit.clone();
                match self.circuit.connect(a, b, p) {
                    Ok(()) => {
                        if old != self.circuit {
                            self.undo.push(old);
                            if self.undo.len()
                                > if self.circuit.nodes.len() > 10000 {
                                    6
                                } else {
                                    80
                                }
                            {
                                self.undo.remove(0);
                            }
                            self.redo.clear();
                        }
                        self.pending_pin = None;
                        self.message(tr("Контакты соединены"), false);
                    }
                    Err(e) => self.message(e, true),
                }
            } else {
                self.pending_pin = Some(pin);
            }
        } else {
            self.pending_pin = Some(pin);
            self.tool = None;
            self.message(tr("Выбран первый контакт"), false);
        }
    }
    fn canvas(&mut self, ctx: &egui::Context, _eval: &Evaluation) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(paper()))
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
                self.canvas_rect = rect;
                if self.fit_next {
                    self.fit(rect);
                    self.fit_next = false;
                }
                let pointer = response.hover_pos();
                let blocked = self.confirm.is_some()
                    || self.truth.is_some()
                    || self.rename.is_some()
                    || self.help_open;
                if !blocked {
                    if let Some(p) = pointer {
                        let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                        if scroll != 0.0 {
                            let before = self.world(p, rect);
                            self.zoom = (self.zoom * (scroll * 0.002).exp()).clamp(0.005, 2.5);
                            self.pan = p - rect.min - before.to_vec2() * self.zoom;
                        }
                    }
                    if self.zoom >= 0.3 && response.drag_started_by(PointerButton::Primary) {
                        if let Some(p) = ctx.input(|i| i.pointer.press_origin()) {
                            if self.pin_at(p, rect).is_none() {
                                self.drag_node = self
                                    .circuit
                                    .nodes
                                    .iter()
                                    .rev()
                                    .find(|n| self.node_rect(n, rect).contains(p))
                                    .map(|n| n.id);
                                if let Some(id) = self.drag_node {
                                    self.checkpoint();
                                    let n = self.circuit.node(id).unwrap();
                                    self.drag_origin = Some((pos2(n.x, n.y), self.world(p, rect)));
                                    self.selected = Some(id);
                                    self.selected_wire = None;
                                }
                            }
                        }
                    }
                    // Use absolute movement from the original grab point. Rounding
                    // per-frame deltas loses small movements and makes slow drags stick.
                    if let (Some(id), Some((origin, grab)), Some(pointer)) = (
                        self.drag_node,
                        self.drag_origin,
                        ctx.input(|i| i.pointer.interact_pos()),
                    ) {
                        if ctx.input(|i| {
                            i.pointer.primary_down()
                                || i.pointer.button_released(PointerButton::Primary)
                        }) {
                            let position =
                                drag_position(origin, grab, self.world(pointer, rect), self.snap);
                            if let Some(n) = self.circuit.nodes.iter_mut().find(|n| n.id == id) {
                                n.x = position.x;
                                n.y = position.y;
                            }
                        }
                    } else if response.dragged_by(PointerButton::Primary) {
                        self.pan += ctx.input(|i| i.pointer.delta());
                    }
                    if response.dragged_by(PointerButton::Middle)
                        || response.dragged_by(PointerButton::Secondary)
                    {
                        self.pan += ctx.input(|i| i.pointer.delta());
                    }
                    // A global release also covers releasing outside the canvas.
                    if ctx.input(|i| i.pointer.button_released(PointerButton::Primary))
                        || response.drag_stopped_by(PointerButton::Primary)
                    {
                        self.drag_node = None;
                        self.drag_origin = None;
                    }
                    if response.clicked_by(PointerButton::Primary) {
                        if let Some(p) = pointer {
                            if self.zoom < 0.3 {
                                let closest = self
                                    .circuit
                                    .nodes
                                    .iter()
                                    .filter_map(|n| {
                                        let d = self.node_rect(n, rect).center().distance(p);
                                        (d < 20.0).then_some((n.id, d))
                                    })
                                    .min_by(|a, b| a.1.total_cmp(&b.1));
                                if let Some((id, _)) = closest {
                                    self.focus_gate(id);
                                }
                            } else if let Some(id) = self.label_at(p, rect) {
                                let n = self.circuit.node(id).unwrap();
                                self.rename = Some(Rename {
                                    id,
                                    draft: if n.label.is_empty() {
                                        n.kind.name().into()
                                    } else {
                                        n.label.clone()
                                    },
                                    focus: true,
                                });
                                self.selected = Some(id);
                                self.selected_wire = None;
                                self.pending_pin = None;
                                self.tool = None;
                            } else if let Some(pin) = self.pin_at(p, rect) {
                                self.connect_pin(pin);
                            } else if let Some(n) = self
                                .circuit
                                .nodes
                                .iter()
                                .rev()
                                .find(|n| self.node_rect(n, rect).contains(p))
                                .cloned()
                            {
                                self.selected = Some(n.id);
                                self.selected_wire = None;
                                if self.pending_pin.is_none() && n.kind == Kind::Input {
                                    self.checkpoint();
                                    self.circuit
                                        .nodes
                                        .iter_mut()
                                        .find(|x| x.id == n.id)
                                        .unwrap()
                                        .value = n.value.flipped();
                                }
                            } else if let Some(kind) = self.tool {
                                let world = placement_position(self.world(p, rect), self.snap);
                                if world.x.abs() <= 100_000.0 && world.y.abs() <= 100_000.0 {
                                    self.checkpoint();
                                    match self.circuit.add(kind, world.x, world.y) {
                                        Ok(id) => {
                                            self.selected = Some(id);
                                            self.selected_wire = None;
                                            self.tool = None;
                                            self.message(tr("Элемент добавлен"), false);
                                        }
                                        Err(e) => self.message(e, true),
                                    }
                                }
                            } else {
                                self.selected = None;
                                self.pending_pin = None;
                                self.selected_wire = self.wire_at(p, rect);
                            }
                        }
                    }
                    if response.clicked_by(PointerButton::Secondary) {
                        if let Some(p) = pointer {
                            let wire = match self.pin_at(p, rect) {
                                Some(Pin::In(id, port)) => self
                                    .circuit
                                    .wires
                                    .iter()
                                    .position(|w| w.to == id && w.port == port),
                                _ => self.wire_at(p, rect),
                            };
                            if let Some(i) = wire {
                                self.checkpoint();
                                self.circuit.wires.remove(i);
                                self.selected_wire = None;
                                self.message(tr("Провод удалён"), false);
                            }
                            self.pending_pin = None;
                            self.tool = None;
                        }
                    }
                }
                let painter = ui.painter_at(rect);
                if self.grid {
                    let mut step = GRID_STEP * self.zoom;
                    while step < 8.0 {
                        step *= 5.0;
                    }
                    let origin = rect.min + self.pan;
                    for vertical in [true, false] {
                        let (low, high, offset) = if vertical {
                            (rect.left(), rect.right(), origin.x)
                        } else {
                            (rect.top(), rect.bottom(), origin.y)
                        };
                        let first = ((low - offset) / step).floor() as i32;
                        let last = ((high - offset) / step).ceil() as i32;
                        for index in first..=last {
                            let position = offset + index as f32 * step;
                            let major = index.rem_euclid(5) == 0;
                            let color = if major {
                                tint(Color32::from_rgb(194, 212, 201))
                            } else {
                                tint(Color32::from_rgb(228, 237, 231))
                            };
                            let points = if vertical {
                                [pos2(position, rect.top()), pos2(position, rect.bottom())]
                            } else {
                                [pos2(rect.left(), position), pos2(rect.right(), position)]
                            };
                            painter.line_segment(
                                points,
                                Stroke::new(if major { 1.0_f32 } else { 0.6_f32 }, color),
                            );
                        }
                    }
                }
                let fresh = self.cached_evaluation();
                if self.zoom >= 0.12 {
                    for (i, w) in self.circuit.wires.iter().enumerate() {
                        if let (Some(a), Some(b)) = (
                            self.pin_pos(Pin::Out(w.from), rect),
                            self.pin_pos(Pin::In(w.to, w.port), rect),
                        ) {
                            if self.circuit.nodes.len() > 10000
                                && self.selected_wire != Some(i)
                                && self.selected != Some(w.from)
                                && self.selected != Some(w.to)
                                && !rect.contains(a)
                                && !rect.contains(b)
                            {
                                continue;
                            }
                            if !wire_bounds(a, b).intersects(rect) {
                                continue;
                            }
                            let value = fresh.values.get(&w.from).copied().flatten();
                            let selected = self.selected_wire == Some(i);
                            if selected {
                                draw_wire(
                                    &painter,
                                    a,
                                    b,
                                    Stroke::new(
                                        7.0 * self.zoom,
                                        tint(Color32::from_rgb(202, 229, 211)),
                                    ),
                                );
                            }
                            wire_animation::draw_signal_wire(
                                &painter,
                                a,
                                b,
                                Stroke::new(
                                    2.6 * self.zoom,
                                    if selected {
                                        accent()
                                    } else {
                                        signal_color(value)
                                    },
                                ),
                                self.animate_wires && value == Some(Bit::One),
                                ctx.input(|i| i.time),
                                w.from.wrapping_mul(17).wrapping_add(w.to) as f64,
                            );
                        }
                    }
                }
                if let Some(pin) = self.pending_pin {
                    if let (Some(a), Some(p)) = (self.pin_pos(pin, rect), pointer) {
                        let (a, b) = if matches!(pin, Pin::Out(_)) {
                            (a, p)
                        } else {
                            (p, a)
                        };
                        draw_wire(&painter, a, b, Stroke::new(2.0 * self.zoom, accent()));
                    }
                }
                let hover_pin = pointer.and_then(|p| self.pin_at(p, rect));
                for n in &self.circuit.nodes {
                    self.paint_node(&painter, n, rect, &fresh, hover_pin);
                }
                if self.zoom < 0.12 {
                    if let Some(layout) = &self.circuit.netlist {
                        for section in &layout.sections {
                            let compact = self.zoom < 0.02;
                            if compact
                                && matches!(
                                    section.name.as_str(),
                                    "ADD" | "SUB" | "SELECT" | "Выходы"
                                )
                            {
                                continue;
                            }
                            if let Some(n) = self.indexed_node(section.anchor) {
                                let p = self.screen(pos2(n.x, n.y), rect);
                                painter.text(
                                    p - vec2(0.0, 8.0),
                                    Align2::LEFT_BOTTOM,
                                    if compact && section.name == "Входы" {
                                        tr("Входы · ADD · SUB · SELECT · Выходы")
                                    } else {
                                        tr(&section.name)
                                    },
                                    FontId::proportional(13.0),
                                    ink(),
                                );
                            }
                        }
                    }
                }
                if self.circuit.nodes.is_empty() {
                    painter.text(
                        rect.center() - vec2(0.0, 20.0),
                        Align2::CENTER_CENTER,
                        tr("Пустая схема"),
                        FontId::proportional(21.0),
                        muted(),
                    );
                }
                if self.tool.is_some() {
                    if let Some(p) = pointer {
                        if rect.contains(p) {
                            painter.rect_stroke(
                                Rect::from_min_size(
                                    self.screen(
                                        placement_position(self.world(p, rect), self.snap),
                                        rect,
                                    ),
                                    NODE_SIZE * self.zoom,
                                ),
                                12.0,
                                Stroke::new(1.5_f32, accent()),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                } else if pointer.is_some_and(|p| self.label_at(p, rect).is_some()) {
                    ctx.set_cursor_icon(egui::CursorIcon::Text);
                } else if hover_pin.is_some() {
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                }
            });
    }
    fn label_at(&self, p: Pos2, rect: Rect) -> Option<NodeId> {
        if self.zoom < 0.3 {
            return None;
        }
        self.circuit
            .nodes
            .iter()
            .rev()
            .find(|n| {
                let r = self.node_rect(n, rect);
                let label = Rect::from_center_size(
                    r.center_top() - vec2(0.0, 16.0 * self.zoom),
                    vec2(155.0, 26.0) * self.zoom,
                );
                let kind = Rect::from_min_max(
                    r.min + vec2(24.0, 64.0) * self.zoom,
                    r.min + vec2(96.0, 80.0) * self.zoom,
                );
                label.contains(p)
                    || (!matches!(n.kind, Kind::Input | Kind::Output) && kind.contains(p))
            })
            .map(|n| n.id)
    }
    fn wire_at(&self, p: Pos2, rect: Rect) -> Option<usize> {
        if self.zoom < 0.12 {
            return None;
        }
        self.circuit
            .wires
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, w)| {
                let a = self.pin_pos(Pin::Out(w.from), rect)?;
                let b = self.pin_pos(Pin::In(w.to, w.port), rect)?;
                if !wire_bounds(a, b).expand(7.0).contains(p) {
                    return None;
                }
                if self.circuit.nodes.len() > 10000
                    && self.selected_wire != Some(i)
                    && self.selected != Some(w.from)
                    && self.selected != Some(w.to)
                    && !rect.contains(a)
                    && !rect.contains(b)
                {
                    return None;
                }
                let points = wire_points(a, b);
                points
                    .windows(2)
                    .any(|s| segment_distance(p, s[0], s[1]) < 7.0)
                    .then_some(i)
            })
    }
    fn paint_node(
        &self,
        painter: &egui::Painter,
        n: &Node,
        rect: Rect,
        eval: &Evaluation,
        hover: Option<Pin>,
    ) {
        let r = self.node_rect(n, rect);
        if !rect.intersects(r.expand(40.0)) {
            return;
        }
        let z = self.zoom;
        let selected = self.selected == Some(n.id);
        let value = eval.values.get(&n.id).copied().flatten();
        let color = signal_color(value);
        if z < 0.12 {
            painter.rect_filled(r, 0.0, color);
            return;
        }
        if self.show_frames {
            painter.rect_filled(
                r.translate(vec2(0.0, 2.0 * z)),
                (12.0 * z).clamp(3.0, 30.0),
                Color32::from_rgba_unmultiplied(35, 75, 48, 9),
            );
            painter.rect_filled(
                r,
                (12.0 * z).clamp(3.0, 30.0),
                if selected {
                    tint(Color32::from_rgb(232, 246, 236))
                } else {
                    panel()
                },
            );
            painter.rect_stroke(
                r,
                (12.0 * z).clamp(3.0, 30.0),
                Stroke::new(
                    if selected { 1.7_f32 } else { 1.0_f32 },
                    if selected { accent() } else { border() },
                ),
                egui::StrokeKind::Outside,
            );
        }
        let at = |x: f32, y: f32| r.min + vec2(x, y - 2.0) * z;
        if n.kind == Kind::Input || n.kind == Kind::Output {
            let center = at(60.0, 42.0);
            painter.circle_filled(
                center,
                23.0 * z,
                if value == Some(Bit::One) {
                    tint(Color32::from_rgb(229, 245, 238))
                } else {
                    tint(Color32::from_rgb(238, 244, 240))
                },
            );
            painter.circle_stroke(center, 23.0 * z, Stroke::new(1.6 * z, color));
            painter.text(
                center,
                Align2::CENTER_CENTER,
                bit_text(value),
                FontId::proportional(25.0 * z),
                color,
            );
            if n.kind == Kind::Input {
                painter.line_segment(
                    [at(84.0, 42.0), at(113.0, 42.0)],
                    Stroke::new(1.8 * z, color),
                );
            } else {
                painter.line_segment([at(7.0, 42.0), at(36.0, 42.0)], Stroke::new(1.8 * z, color));
            }
        } else {
            let stroke = Stroke::new(
                if selected { 2.4 * z } else { 1.8 * z },
                if selected { accent() } else { ink() },
            );
            let inverted = matches!(n.kind, Kind::Not | Kind::Nand | Kind::Nor | Kind::Xnor);
            match n.kind {
                Kind::And | Kind::Nand => {
                    painter.line_segment([at(34.0, 19.0), at(34.0, 65.0)], stroke);
                    painter.line_segment([at(34.0, 19.0), at(59.0, 19.0)], stroke);
                    painter.line_segment([at(34.0, 65.0), at(59.0, 65.0)], stroke);
                    let points = (0..=32)
                        .map(|i| {
                            let t = -std::f32::consts::FRAC_PI_2
                                + std::f32::consts::PI * i as f32 / 32.0;
                            at(59.0 + 24.0 * t.cos(), 42.0 + 23.0 * t.sin())
                        })
                        .collect();
                    painter.add(egui::Shape::line(points, stroke));
                }
                Kind::Not | Kind::Buffer => {
                    painter.add(egui::Shape::closed_line(
                        vec![at(34.0, 19.0), at(34.0, 65.0), at(82.0, 42.0)],
                        stroke,
                    ));
                }
                _ => {
                    for points in [
                        [
                            at(30.0, 19.0),
                            at(59.0, 18.0),
                            at(76.0, 24.0),
                            at(88.0, 42.0),
                        ],
                        [
                            at(88.0, 42.0),
                            at(76.0, 60.0),
                            at(59.0, 66.0),
                            at(30.0, 65.0),
                        ],
                        [
                            at(30.0, 65.0),
                            at(44.0, 49.0),
                            at(44.0, 35.0),
                            at(30.0, 19.0),
                        ],
                    ] {
                        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                            points,
                            false,
                            Color32::TRANSPARENT,
                            stroke,
                        ));
                    }
                    if matches!(n.kind, Kind::Xor | Kind::Xnor) {
                        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                            [
                                at(23.0, 65.0),
                                at(37.0, 49.0),
                                at(37.0, 35.0),
                                at(23.0, 19.0),
                            ],
                            false,
                            Color32::TRANSPARENT,
                            stroke,
                        ));
                    }
                }
            }
            let body_end = if matches!(n.kind, Kind::And | Kind::Nand | Kind::Not | Kind::Buffer) {
                83.0
            } else {
                88.0
            };
            if inverted {
                painter.circle_stroke(at(body_end + 5.0, 42.0), 4.5 * z, stroke);
            }
            painter.line_segment(
                [
                    at(body_end + if inverted { 10.0 } else { 0.0 }, 42.0),
                    at(113.0, 42.0),
                ],
                stroke,
            );
            for port in 0..n.kind.inputs() {
                let y = if n.kind.inputs() == 1 {
                    42.0
                } else if port == 0 {
                    22.0
                } else {
                    62.0
                };
                painter.line_segment(
                    [
                        at(7.0, y),
                        at(
                            if matches!(n.kind, Kind::Xor | Kind::Xnor) {
                                28.0
                            } else {
                                34.0
                            },
                            y,
                        ),
                    ],
                    stroke,
                );
            }
            painter.text(
                at(60.0, 74.0),
                Align2::CENTER_CENTER,
                n.kind.name(),
                FontId::proportional(9.0 * z),
                muted(),
            );
        }
        {
            let name = if n.label.is_empty() {
                n.kind.name()
            } else {
                &n.label
            };
            let label = if name.chars().count() > 19 {
                format!("{}…", name.chars().take(18).collect::<String>())
            } else {
                name.to_owned()
            };
            painter.text(
                at(60.0, -14.0),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(13.0 * z),
                if selected { accent() } else { ink() },
            );
        }
        let pins = (0..n.kind.inputs())
            .map(|p| Pin::In(n.id, p))
            .chain(n.kind.has_output().then_some(Pin::Out(n.id)));
        for pin in pins {
            let p = self.pin_pos(pin, rect).unwrap();
            let input_value = match pin {
                Pin::Out(_) => value,
                Pin::In(id, port) => self
                    .sources
                    .get(&(id, port))
                    .and_then(|from| eval.values.get(from).copied().flatten()),
            };
            let active = hover == Some(pin) || self.pending_pin == Some(pin);
            if active {
                painter.circle_filled(p, 10.0 * z, tint(Color32::from_rgb(211, 237, 220)));
            }
            painter.circle_filled(p, 5.5 * z, panel());
            painter.circle_stroke(
                p,
                5.5 * z,
                Stroke::new(
                    2.0 * z,
                    if active {
                        accent()
                    } else {
                        signal_color(input_value)
                    },
                ),
            );
        }
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        self.help_dialog(ctx);
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.confirm = None;

            self.truth = None;
            self.rename = None;
        }
        if let Some(mut rename) = self.rename.take() {
            let mut accept = false;
            let mut cancel = false;
            egui::Modal::new(Id::new("rename_element")).show(ctx, |ui| {
                ui.set_width(380.0);
                ui.heading(tr("Название элемента"));
                ui.label(tr("Введите новую подпись на схеме."));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut rename.draft)
                        .char_limit(48)
                        .desired_width(f32::INFINITY),
                );
                if rename.focus {
                    response.request_focus();
                    rename.focus = false;
                }
                accept = ctx.input(|i| i.key_pressed(Key::Enter));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("Применить")).clicked() {
                        accept = true;
                    }
                    if ui.button(tr("Отмена")).clicked() {
                        cancel = true;
                    }
                });
            });
            if accept {
                let label = rename.draft.trim().to_owned();
                if self
                    .circuit
                    .node(rename.id)
                    .is_some_and(|n| n.label != label)
                {
                    self.checkpoint();
                    self.circuit
                        .nodes
                        .iter_mut()
                        .find(|n| n.id == rename.id)
                        .unwrap()
                        .label = label;
                    self.message(tr("Название элемента изменено"), false);
                }
            } else if !cancel {
                self.rename = Some(rename);
            }
        }
        if let Some(action) = self.confirm {
            egui::Modal::new(Id::new("unsaved")).show(ctx, |ui| {
                ui.set_width(420.0);
                ui.heading(tr("Сохранить изменения?"));
                ui.label(tr(
                    "В текущей схеме есть изменения, которые ещё не записаны в файл.",
                ));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("Сохранить")).clicked() && self.save(false) {
                        self.confirm = None;
                        self.perform(action);
                    }
                    if ui.button(tr("Не сохранять")).clicked() {
                        self.confirm = None;
                        self.perform(action);
                    }
                    if ui.button(tr("Отмена")).clicked() {
                        self.confirm = None;
                    }
                });
            });
        }
        if let Some(table) = self.truth.clone() {
            egui::Modal::new(Id::new("truth")).show(ctx, |ui| {
                ui.set_width(580.0);
                ui.heading(tr("Таблица истинности"));
                ui.label(
                    RichText::new(tr("Все сочетания входных битов и вычисленные выходы"))
                        .size(12.0)
                        .color(muted()),
                );
                ui.separator();
                match &table {
                    Err(e) => {
                        ui.colored_label(error(), e);
                    }
                    Ok(table) => {
                        egui::ScrollArea::both().max_height(450.0).show(ui, |ui| {
                            egui::Grid::new("truth_grid")
                                .striped(true)
                                .min_col_width(65.0)
                                .spacing(vec2(20.0, 10.0))
                                .show(ui, |ui| {
                                    for (_, name) in table.inputs.iter().chain(&table.outputs) {
                                        ui.label(
                                            RichText::new(if name.is_empty() {
                                                tr("Бит")
                                            } else {
                                                name
                                            })
                                            .strong(),
                                        );
                                    }
                                    ui.end_row();
                                    for (input, output) in &table.rows {
                                        for bit in input.iter().chain(output) {
                                            ui.label(
                                                RichText::new(bit.text())
                                                    .monospace()
                                                    .color(signal_color(Some(*bit))),
                                            );
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                        if ui.button(tr("Скопировать таблицу")).clicked() {
                            let mut t = table
                                .inputs
                                .iter()
                                .chain(&table.outputs)
                                .map(|(_, s)| s.as_str())
                                .collect::<Vec<_>>()
                                .join("\t");
                            for (a, b) in &table.rows {
                                t.push('\n');
                                t.push_str(
                                    &a.iter()
                                        .chain(b)
                                        .map(|v| v.text())
                                        .collect::<Vec<_>>()
                                        .join("\t"),
                                );
                            }
                            ctx.copy_text(t);
                        }
                    }
                }
                if ui.button(tr("Закрыть")).clicked() {
                    self.truth = None;
                }
            });
        }
    }
}

impl Editor {
    pub fn ui(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close {
            if self.dirty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.confirm = Some(Action::Quit);
            } else {
                self.allow_close = true;
            }
        }
        if self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.prepare_preferences(ctx);
        self.shortcuts(ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            if xor_112::i18n::is_english() {
                format!(
                    "{}{} — XOR-112",
                    self.circuit.title,
                    if self.dirty() { " *" } else { "" }
                )
            } else {
                format!(
                    "{}{} — XOR-112",
                    self.circuit.title,
                    if self.dirty() { " *" } else { "" }
                )
            },
        ));
        self.toolbar(ctx);
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(panel())
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                let (row, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
                let controls = Rect::from_min_max(pos2(row.right() - 260.0, row.top()), row.max);
                let status =
                    Rect::from_min_max(row.min, pos2(controls.left() - 16.0, row.bottom()));
                ui.put(
                    status,
                    egui::Label::new(
                        RichText::new(xor_112::i18n::display_message(&self.status))
                            .size(12.0)
                            .color(if self.status_error { error() } else { muted() }),
                    )
                    .truncate(),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(controls)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        if ui
                            .add_sized([32.0, 32.0], egui::Button::new("−").corner_radius(9))
                            .clicked()
                        {
                            self.set_zoom(self.zoom / 1.15);
                        }
                        self.zoom_control(ui, "status_zoom");
                        if ui
                            .add_sized([32.0, 32.0], egui::Button::new("+").corner_radius(9))
                            .clicked()
                        {
                            self.set_zoom(self.zoom * 1.15);
                        }
                        if ui
                            .add_sized(
                                [104.0, 32.0],
                                egui::Button::new(tr("Вся схема")).corner_radius(9),
                            )
                            .clicked()
                        {
                            self.fit_next = true;
                        }
                    },
                );
            });
        if self.circuit.alu64.is_some() {
            self.alu_panel(ctx);
            self.alu_canvas(ctx);
        } else {
            let eval = self.cached_evaluation();
            self.palette(ctx);
            self.inspector(ctx, &eval);
            self.canvas(ctx, &eval);
        }
        self.dialogs(ctx);
        self.persist_preferences(ctx);
        // Immediate-mode painting can display a just-edited value one frame late.
        if ctx.input(|i| i.pointer.any_released() || !i.events.is_empty()) {
            ctx.request_repaint();
        }
    }
}

fn signal_color(value: Option<Bit>) -> Color32 {
    match value {
        Some(Bit::One) => high(),
        Some(Bit::Zero) => low(),
        None => error(),
    }
}
fn bit_text(value: Option<Bit>) -> &'static str {
    value.map(Bit::text).unwrap_or("—")
}
fn wire_controls(a: Pos2, b: Pos2) -> [Pos2; 4] {
    let bend = ((b.x - a.x).abs() * 0.5).clamp(45.0, 200.0);
    [a, a + vec2(bend, 0.0), b - vec2(bend, 0.0), b]
}
fn draw_wire(painter: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke) {
    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        wire_controls(a, b),
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
}
fn wire_points(a: Pos2, b: Pos2) -> Vec<Pos2> {
    let [a, b, c, d] = wire_controls(a, b);
    (0..=40)
        .map(|i| {
            let t = i as f32 / 40.0;
            let u = 1.0 - t;
            (a.to_vec2() * u * u * u
                + b.to_vec2() * 3.0 * u * u * t
                + c.to_vec2() * 3.0 * u * t * t
                + d.to_vec2() * t * t * t)
                .to_pos2()
        })
        .collect()
}
fn segment_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(0.0001)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

fn ribbon_caption(ui: &mut egui::Ui, title: &str, width: f32) {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 18.0), Sense::hover());
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(11.0),
        muted(),
    );
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), title));
}

impl eframe::App for Editor {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.flush_preferences();
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);
    }
}

fn snap_position(position: Pos2, enabled: bool) -> Pos2 {
    let bounded = pos2(
        position.x.clamp(-100_000.0, 100_000.0),
        position.y.clamp(-100_000.0, 100_000.0),
    );
    if enabled {
        pos2(
            (bounded.x / GRID_STEP).round() * GRID_STEP,
            (bounded.y / GRID_STEP).round() * GRID_STEP,
        )
    } else {
        bounded
    }
}
fn placement_position(pointer: Pos2, snap: bool) -> Pos2 {
    snap_position(pointer - NODE_SIZE * 0.5, snap)
}
fn drag_position(origin: Pos2, grab: Pos2, pointer: Pos2, snap: bool) -> Pos2 {
    snap_position(origin + (pointer - grab), snap)
}

#[cfg(test)]
mod ui_tests {
    use super::*;
    use egui::{Event, Modifiers, RawInput};

    fn frame(
        app: &mut Editor,
        ctx: &egui::Context,
        time: &mut f64,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        *time += 0.02;
        ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 800.0))),
                time: Some(*time),
                events,
                ..Default::default()
            },
            |ctx| app.ui(ctx),
        )
    }
    fn fixture() -> (Editor, egui::Context, f64) {
        let ctx = egui::Context::default();
        let mut app = Editor::from_context(&ctx);
        let mut time = 0.0;
        frame(&mut app, &ctx, &mut time, vec![]);
        frame(&mut app, &ctx, &mut time, vec![]);
        (app, ctx, time)
    }
    fn pointer(p: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }
    fn click(app: &mut Editor, ctx: &egui::Context, time: &mut f64, p: Pos2) {
        frame(app, ctx, time, vec![Event::PointerMoved(p)]);
        frame(app, ctx, time, vec![pointer(p, true)]);
        frame(app, ctx, time, vec![pointer(p, false)]);
    }
    #[test]
    fn drag_snaps_live_at_different_zoom_and_releases_outside_canvas() {
        for zoom in [0.5, 1.0, 1.6] {
            let (mut app, ctx, mut time) = fixture();
            app.zoom = zoom;
            app.pan = vec2(20.0, 70.0);
            app.fit_next = false;
            frame(&mut app, &ctx, &mut time, vec![]);
            let before = app.circuit.clone();
            let start = app
                .node_rect(app.circuit.node(3).unwrap(), app.canvas_rect)
                .center();
            frame(&mut app, &ctx, &mut time, vec![Event::PointerMoved(start)]);
            frame(&mut app, &ctx, &mut time, vec![pointer(start, true)]);
            for delta in [vec2(13.0, 12.0), vec2(23.0, 21.0), vec2(37.0, 29.0)] {
                frame(
                    &mut app,
                    &ctx,
                    &mut time,
                    vec![Event::PointerMoved(start + delta * zoom)],
                );
            }
            let n = app.circuit.node(3).unwrap();
            assert_eq!((n.x, n.y), (380.0, 120.0), "live snap at zoom {zoom}");
            let outside = app.canvas_rect.right_bottom() + vec2(25.0, 12.0);
            frame(
                &mut app,
                &ctx,
                &mut time,
                vec![Event::PointerMoved(outside), pointer(outside, false)],
            );
            assert!(app.drag_node.is_none());
            let n = app.circuit.node(3).unwrap();
            assert_eq!(n.x.rem_euclid(GRID_STEP), 0.0);
            assert_eq!(n.y.rem_euclid(GRID_STEP), 0.0);
            assert_eq!(app.undo.len(), 1);
            app.undo();
            assert_eq!(app.circuit, before);
        }
    }
    #[test]
    fn snap_can_be_disabled_and_small_movements_accumulate() {
        let (mut app, ctx, mut time) = fixture();
        app.snap = false;
        app.zoom = 1.0;
        app.pan = vec2(20.0, 70.0);
        app.fit_next = false;
        frame(&mut app, &ctx, &mut time, vec![]);
        let start = app
            .node_rect(app.circuit.node(3).unwrap(), app.canvas_rect)
            .center();
        frame(
            &mut app,
            &ctx,
            &mut time,
            vec![Event::PointerMoved(start), pointer(start, true)],
        );
        for i in 1..=37 {
            frame(
                &mut app,
                &ctx,
                &mut time,
                vec![Event::PointerMoved(start + vec2(i as f32, 0.0))],
            );
        }
        let n = app.circuit.node(3).unwrap();
        assert_eq!(n.x, 377.0);
        frame(
            &mut app,
            &ctx,
            &mut time,
            vec![pointer(start + vec2(37.0, 0.0), false)],
        );
        assert_eq!(app.circuit.node(3).unwrap().x, 377.0);
    }
    #[test]
    fn placement_and_all_pins_align_with_grid() {
        let (mut app, ctx, mut time) = fixture();
        app.zoom = 0.8;
        app.pan = vec2(20.0, 70.0);
        app.fit_next = false;
        app.tool = Some(Kind::Not);
        frame(&mut app, &ctx, &mut time, vec![]);
        let p = app.screen(pos2(503.0, 461.0), app.canvas_rect);
        click(&mut app, &ctx, &mut time, p);
        let n = app.circuit.nodes.last().unwrap();
        assert_eq!(n.kind, Kind::Not);
        assert_eq!((n.x, n.y), (440.0, 420.0));
        for n in &app.circuit.nodes {
            for pin in (0..n.kind.inputs())
                .map(|p| Pin::In(n.id, p))
                .chain(n.kind.has_output().then_some(Pin::Out(n.id)))
            {
                let p = app.world(app.pin_pos(pin, app.canvas_rect).unwrap(), app.canvas_rect);
                assert!((p.x / GRID_STEP - (p.x / GRID_STEP).round()).abs() < 0.0001);
                assert!((p.y / GRID_STEP - (p.y / GRID_STEP).round()).abs() < 0.0001);
            }
        }
    }
    #[test]
    fn clicking_a_name_renames_without_toggling_the_input() {
        let (mut app, ctx, mut time) = fixture();
        let value = app.circuit.node(1).unwrap().value;
        let p = app
            .node_rect(app.circuit.node(1).unwrap(), app.canvas_rect)
            .center_top()
            - vec2(0.0, 16.0 * app.zoom);
        click(&mut app, &ctx, &mut time, p);
        assert_eq!(app.rename.as_ref().unwrap().id, 1);
        assert_eq!(app.circuit.node(1).unwrap().value, value);
        app.rename.as_mut().unwrap().draft = "Вход A".into();
        frame(
            &mut app,
            &ctx,
            &mut time,
            vec![Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert_eq!(app.circuit.node(1).unwrap().label, "Вход A");
        app.undo();
        assert_eq!(app.circuit.node(1).unwrap().label, "A");
    }
    #[test]
    fn selected_palette_is_rounded_and_cards_can_be_hidden() {
        let (mut app, ctx, mut time) = fixture();
        assert!(app.show_frames);
        app.tool = Some(Kind::And);
        app.selected = Some(3);
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let selected: Vec<_> = out
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Rect(r) = &s.shape {
                    Some(r)
                } else {
                    None
                }
            })
            .filter(|r| {
                r.rect.left() < 204.0
                    && r.rect.top() > app.canvas_rect.top()
                    && r.fill == ctx.style().visuals.selection.bg_fill
            })
            .collect();
        assert!(!selected.is_empty());
        assert!(selected
            .iter()
            .all(|r| r.corner_radius == egui::CornerRadius::same(9)));
        let node_rect = app.node_rect(app.circuit.node(3).unwrap(), app.canvas_rect);
        let has_card = |out: &egui::FullOutput| {
            out.shapes
                .iter()
                .any(|s| matches!(&s.shape,egui::Shape::Rect(r) if r.rect==node_rect))
        };
        assert!(has_card(&out));
        app.show_frames = false;
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        assert!(!has_card(&out));
    }
    #[test]
    fn render_ui_previews_when_requested() {
        let Some(dir) = std::env::var_os("BINARY_LOGIC_RENDER_DIR") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (name, frames) in [
            ("home", true),
            ("no-frames", false),
            ("alu64", true),
            ("alu-detail", true),
            ("help", true),
            ("home-narrow", true),
            ("settings-blue-en", true),
            ("home-blue-en", true),
            ("home-gray", true),
            ("alu32-detail", true),
        ] {
            if let Ok(filter) = std::env::var("BINARY_LOGIC_RENDER_FILTER") {
                if !filter.split(',').any(|item| item == name) {
                    continue;
                }
            }
            let ctx = egui::Context::default();
            let mut app = Editor::from_context(&ctx);
            let mut time = 0.0;
            app.selected = Some(3);
            app.tool = Some(Kind::And);

            app.show_frames = frames;
            app.help_open = name == "help";
            if name.contains("blue") {
                app.language = Language::English;
                app.theme = Theme::Blue;
                xor_112::i18n::set_language(app.language);
                appearance::apply(&ctx, app.theme);
                app.circuit = Circuit::half_adder();
                app.saved = app.circuit.clone();
            }
            if name == "settings-blue-en" {
                app.tab = ribbon::Tab::Settings;
            }
            if name == "home-gray" {
                app.theme = Theme::Gray;
                appearance::apply(&ctx, app.theme);
            }
            if name == "alu64" || name == "alu-detail" {
                app.perform(Action::Alu64);
            }
            if name == "alu32-detail" {
                app.perform(Action::Alu32);
            }
            let mut textures = vec![];
            let mut last = None;
            for pass in 0..3 {
                if name == "help" && pass == 2 {
                    time += 1.0;
                }
                if (name == "alu-detail" || name == "alu32-detail") && pass == 1 {
                    let id = app.circuit.netlist.as_ref().unwrap().sections[1].anchor + 17;
                    app.focus_gate(id);
                }
                let out = if name == "home-narrow" {
                    time += 1.0 / 60.0;
                    ctx.run(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1000., 650.))),
                            time: Some(time),
                            ..Default::default()
                        },
                        |ctx| app.ui(ctx),
                    )
                } else {
                    frame(&mut app, &ctx, &mut time, vec![])
                };
                for (id, delta) in &out.textures_delta.set {
                    let pixels: Vec<[u8; 4]> = match &delta.image {
                        egui::ImageData::Color(image) => {
                            image.pixels.iter().map(|c| c.to_array()).collect()
                        }
                        egui::ImageData::Font(image) => {
                            image.srgba_pixels(None).map(|c| c.to_array()).collect()
                        }
                    };
                    textures.push(serde_json::json!({"id":format!("{id:?}"),"pos":delta.pos,"size":delta.image.size(),"pixels":pixels}));
                }
                last = Some(out);
            }
            let out = last.unwrap();
            let meshes:Vec<_>=ctx.tessellate(out.shapes,out.pixels_per_point).into_iter().filter_map(|p| {
                if let egui::epaint::Primitive::Mesh(m)=p.primitive {
                    Some(serde_json::json!({"clip":[p.clip_rect.min.x,p.clip_rect.min.y,p.clip_rect.max.x,p.clip_rect.max.y],"texture":format!("{:?}",m.texture_id),"indices":m.indices,"vertices":m.vertices.iter().map(|v|serde_json::json!([v.pos.x,v.pos.y,v.uv.x,v.uv.y,v.color.to_array()])).collect::<Vec<_>>()}))
                }else{None}
            }).collect();
            std::fs::write(dir.join(format!("{name}.json")),serde_json::to_vec(&serde_json::json!({"width":if name=="home-narrow"{1000}else{1280},"height":if name=="home-narrow"{650}else{800},"textures":textures,"meshes":meshes})).unwrap()).unwrap();
        }
    }

    #[test]
    fn help_and_display_controls_are_available_without_canvas_hints() {
        let (mut app, ctx, mut time) = fixture();
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        for label in [
            "Сетка",
            "Привязка к сетке",
            "Рамки элементов",
            "Анимация проводов",
        ] {
            let texts: Vec<_> = out
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == label => Some(t),
                    _ => None,
                })
                .collect();
            assert_eq!(texts.len(), 1, "one setting in the ribbon: {label}");
            assert!(texts[0].pos.y < app.canvas_rect.top());
        }
        assert!(!out.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text().contains("Колесо:")||t.galley.text()=="СОБЕРИТЕ СХЕМУ")));
        let label_pos = |out: &egui::FullOutput, label: &str| {
            out.shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + vec2(5., 5.)),
                    _ => None,
                })
                .unwrap()
        };
        let animation = label_pos(&out, "Анимация проводов");
        assert!(app.animate_wires);
        click(&mut app, &ctx, &mut time, animation);
        assert!(!app.animate_wires);
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let p = label_pos(&out, "Справка");
        click(&mut app, &ctx, &mut time, p);
        assert!(app.help_open);
        app.selected = Some(1);
        let before = app.circuit.clone();
        let key = |key| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        frame(&mut app, &ctx, &mut time, vec![key(Key::Delete)]);
        assert_eq!(app.circuit, before);
        frame(&mut app, &ctx, &mut time, vec![key(Key::Escape)]);
        assert!(!app.help_open);
        frame(&mut app, &ctx, &mut time, vec![key(Key::F1)]);
        assert!(app.help_open);
    }

    #[test]
    fn expanded_alu_is_editable_in_the_real_editor_and_cached() {
        let (mut app, ctx, mut time) = fixture();
        app.perform(Action::Alu64);
        let started = std::time::Instant::now();
        frame(&mut app, &ctx, &mut time, vec![]);
        assert!(app.zoom < 0.1);
        assert!(app.circuit.alu64.is_none());
        let l = app.circuit.netlist.clone().unwrap();
        let eval = app.cached_evaluation();
        frame(&mut app, &ctx, &mut time, vec![]);
        assert!(Arc::ptr_eq(&eval, &app.cached_evaluation()));
        let id = l.sections[1].anchor + 17;
        app.focus_gate(id);
        frame(&mut app, &ctx, &mut time, vec![]);
        let n = app.circuit.node(id).unwrap().clone();
        let p = app.screen(pos2(n.x + 60., n.y + 40.), app.canvas_rect);
        frame(&mut app, &ctx, &mut time, vec![pointer(p, true)]);
        frame(
            &mut app,
            &ctx,
            &mut time,
            vec![Event::PointerMoved(p + vec2(32., 16.))],
        );
        frame(
            &mut app,
            &ctx,
            &mut time,
            vec![pointer(p + vec2(32., 16.), false)],
        );
        assert_eq!(app.circuit.node(id).unwrap().x, n.x + 40.);
        assert_eq!(app.circuit.node(id).unwrap().y, n.y + 20.);
        assert!(
            Arc::ptr_eq(&eval, &app.cached_evaluation()),
            "moving gates must not recalculate logic"
        );
        app.undo();
        assert_eq!(app.circuit.node(id).unwrap().x, n.x);
        app.drive_netlist(16, 2, Some(4));
        let e = app.cached_evaluation();
        assert_eq!(e.values[&l.result[2]], Some(Bit::One));
        assert!(!Arc::ptr_eq(&eval, &e));
        app.undo();
        assert_eq!(app.circuit.node(l.a[0]).unwrap().value, Bit::One);
        // Use the same connection handlers as clicks on real pins.
        app.checkpoint();
        app.circuit.wires.retain(|w| w.to != l.result[0]);
        app.connect_pin(Pin::Out(l.a[0]));
        app.focus_gate(l.result[0]);
        app.connect_pin(Pin::In(l.result[0], 0));
        assert_eq!(app.cached_evaluation().values[&l.result[0]], Some(Bit::One));
        eprintln!(
            "Expanded editor open, overview, detail, drag, undo and rewiring: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn settings_survive_restart_and_language_switch_preserves_the_circuit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences/settings.json");
        let (mut app, ctx, mut time) = fixture();
        app.settings_path = Some(path.clone());
        let label_pos = |out: &egui::FullOutput, label: &str| {
            out.shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + vec2(5., 5.)),
                    _ => None,
                })
                .unwrap()
        };
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let p = label_pos(&out, "Настройки");
        click(&mut app, &ctx, &mut time, p);
        assert!(app.tab == ribbon::Tab::Settings);
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let p = label_pos(&out, "English");
        let circuit = app.circuit.clone();
        click(&mut app, &ctx, &mut time, p);
        assert_eq!(app.language, Language::English);
        assert_eq!(app.circuit, circuit);
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let p = label_pos(&out, "Blue");
        click(&mut app, &ctx, &mut time, p);
        assert_eq!(
            ctx.style().visuals.selection.stroke.color,
            Color32::from_rgb(36, 101, 180)
        );
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let p = label_pos(&out, "Home");
        click(&mut app, &ctx, &mut time, p);
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let grid = label_pos(&out, "Grid");
        let animation = label_pos(&out, "Wire animation");
        assert!(animation.x > grid.x + 100. && (animation.y - grid.y).abs() < 3.);
        for label in ["Grid", "Snap to grid", "Element frames", "Wire animation"] {
            let out = frame(&mut app, &ctx, &mut time, vec![]);
            let p = label_pos(&out, label);
            click(&mut app, &ctx, &mut time, p);
        }
        app.set_zoom(0.75);
        app.flush_preferences();
        let expected = app.preferences();
        assert_eq!(xor_112::preferences::load(&path).unwrap(), expected);
        let ctx2 = egui::Context::default();
        let mut restored = Editor::from_context(&ctx2);
        restored.restore_preferences(&ctx2, Some(path));
        frame(&mut restored, &ctx2, &mut time, vec![]);
        assert_eq!(restored.preferences(), expected);
        assert!(
            !restored.grid && !restored.snap && !restored.show_frames && !restored.animate_wires
        );
        let out = frame(&mut restored, &ctx2, &mut time, vec![]);
        assert!(!out.shapes.iter().any(
            |s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="КОМБИНАЦИОННАЯ СХЕМА")
        ));
        // A loaded/user-edited title or label is data, not a UI translation key.
        restored.circuit.nodes[0].label = "Мой вход".into();
        let before = restored.circuit.clone();
        restored.language = Language::Russian;
        restored.theme = Theme::Gray;
        frame(&mut restored, &ctx2, &mut time, vec![]);
        assert_eq!(restored.circuit, before);
        assert_eq!(
            ctx2.style().visuals.selection.stroke.color,
            Color32::from_rgb(91, 100, 109)
        );
    }

    #[test]
    fn zoom_percent_inputs_accept_values_and_reject_invalid_text() {
        assert_eq!(parse_zoom("150%"), Some(1.5));
        assert_eq!(parse_zoom(" 75,5 % "), Some(0.755));
        for invalid in ["", "abc", "NaN", "inf", "0.4", "251", "-50"] {
            assert_eq!(parse_zoom(invalid), None);
        }
        for (control, value, expected) in [(0, "150", 1.5), (1, "75", 0.75)] {
            let (mut app, ctx, mut time) = fixture();
            let out = frame(&mut app, &ctx, &mut time, vec![]);
            let expected_label = format!("{:.0}%", app.zoom * 100.0);
            let mut positions: Vec<_> = out
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == expected_label => {
                        Some(t.pos + vec2(5.0, 5.0))
                    }
                    _ => None,
                })
                .collect();
            positions.sort_by(|a, b| a.y.total_cmp(&b.y));
            assert_eq!(positions.len(), 2);
            click(&mut app, &ctx, &mut time, positions[control]);
            frame(&mut app, &ctx, &mut time, vec![]);
            assert!(app.zoom_edit.is_some());
            frame(&mut app, &ctx, &mut time, vec![Event::Text(value.into())]);
            frame(
                &mut app,
                &ctx,
                &mut time,
                vec![Event::Key {
                    key: Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            assert_eq!(app.zoom, expected);
            assert!(app.zoom_edit.is_none());
        }
    }

    #[test]
    fn alu_example_selects_all_five_blocks_and_applies_binary_inputs() {
        use xor_112::alu64::Operation;
        let (mut app, ctx, mut time) = fixture();
        app.replace(Circuit::legacy_alu64_example(), None);
        frame(&mut app, &ctx, &mut time, vec![]);
        frame(&mut app, &ctx, &mut time, vec![]);
        for (i, (operation, answer)) in [
            (Operation::Add, 12),
            (Operation::Subtract, 6),
            (Operation::Multiply, 27),
            (Operation::Divide, 3),
            (Operation::Sqrt, 3),
        ]
        .into_iter()
        .enumerate()
        {
            let p = app.screen(pos2(430.0, 88.0 + i as f32 * 105.0), app.canvas_rect);
            click(&mut app, &ctx, &mut time, p);
            let alu = app.circuit.alu64.as_ref().unwrap();
            assert_eq!(alu.operation, operation);
            assert_eq!(alu.evaluate().low, answer);
        }
        app.alu_a = "10000".into();
        app.alu_b = "10".into();
        let out = frame(&mut app, &ctx, &mut time, vec![]);
        let p = out
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.text() == "Применить входы" => {
                    Some(t.pos + vec2(5.0, 5.0))
                }
                _ => None,
            })
            .unwrap();
        click(&mut app, &ctx, &mut time, p);
        assert_eq!(app.circuit.alu64.as_ref().unwrap().evaluate().low, 4);
        app.undo();
        assert_eq!(app.circuit.alu64.as_ref().unwrap().a.0, 9);
    }
}

fn parse_zoom(text: &str) -> Option<f32> {
    let percent = text
        .trim()
        .trim_end_matches('%')
        .trim()
        .replace(',', ".")
        .parse::<f32>()
        .ok()?;
    if percent.is_finite() && (0.5..=250.0).contains(&percent) {
        Some(percent / 100.0)
    } else {
        None
    }
}

fn zoom_text(zoom: f32) -> String {
    if zoom < 0.1 {
        format!("{:.1}", zoom * 100.0)
    } else {
        format!("{:.0}", zoom * 100.0)
    }
}
fn wire_bounds(a: Pos2, b: Pos2) -> Rect {
    wire_controls(a, b)
        .into_iter()
        .fold(Rect::NOTHING, |r, p| r.union(Rect::from_min_max(p, p)))
}

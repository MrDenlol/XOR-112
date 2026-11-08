use super::*;
use xor_112::alu64::parse_word;

impl Editor {
    pub(super) fn indexed_node(&self, id: NodeId) -> Option<&Node> {
        self.node_index
            .get(&id)
            .and_then(|&i| self.circuit.nodes.get(i))
            .filter(|n| n.id == id)
            .or_else(|| self.circuit.node(id))
    }
    pub(super) fn cached_evaluation(&mut self) -> Arc<Evaluation> {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.circuit.nodes.len().hash(&mut hash);
        for n in &self.circuit.nodes {
            n.id.hash(&mut hash);
            n.kind.hash(&mut hash);
            n.value.hash(&mut hash);
        }
        for w in &self.circuit.wires {
            w.from.hash(&mut hash);
            w.to.hash(&mut hash);
            w.port.hash(&mut hash);
        }
        let key = hash.finish();
        if let Some((old, eval)) = &self.evaluation {
            if *old == key {
                return eval.clone();
            }
        }
        self.node_index = self
            .circuit
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id, i))
            .collect();
        self.sources = self
            .circuit
            .wires
            .iter()
            .map(|w| ((w.to, w.port), w.from))
            .collect();
        let eval = Arc::new(self.circuit.evaluate().unwrap_or(Evaluation {
            values: HashMap::new(),
            missing: vec![],
        }));
        self.evaluation = Some((key, eval.clone()));
        eval
    }
    pub(super) fn focus_gate(&mut self, id: NodeId) {
        if let Some(n) = self.circuit.node(id) {
            let point = pos2(n.x, n.y) + NODE_SIZE * 0.5;
            self.zoom = 0.8;
            self.pan = self.canvas_rect.size() * 0.5 - point.to_vec2() * self.zoom;
            self.fit_next = false;
            self.selected = Some(id);
            self.selected_wire = None;
            self.tool = None;
        } else {
            self.message(tr("Этот элемент удалён из схемы"), true);
        }
    }
    pub(super) fn sync_netlist_inputs(&mut self) {
        if let Some(l) = &self.circuit.netlist {
            let word = |ids: &[NodeId]| {
                ids.iter().enumerate().fold(0u64, |v, (i, id)| {
                    v | ((self.circuit.node(*id).is_some_and(|n| n.value.is_one()) as u64) << i)
                })
            };
            self.alu_a = format_bus(word(&l.a), l.bits);
            self.alu_b = format_bus(word(&l.b), l.bits);
        }
    }
    pub(super) fn drive_netlist(&mut self, a: u64, b: u64, op: Option<u64>) {
        let Some(l) = self.circuit.netlist.clone() else {
            return;
        };
        self.checkpoint();
        let mut values = HashMap::new();
        for (ids, word) in [(&l.a, a), (&l.b, b)] {
            for (i, &id) in ids.iter().enumerate() {
                values.insert(id, Bit::from_bool(word >> i & 1 != 0));
            }
        }
        if let Some(op) = op {
            for (i, &id) in l.op.iter().enumerate() {
                values.insert(id, Bit::from_bool(op >> i & 1 != 0));
            }
        }
        for n in &mut self.circuit.nodes {
            if n.kind == Kind::Input {
                if let Some(value) = values.get(&n.id) {
                    n.value = *value;
                }
            }
        }
        self.sync_netlist_inputs();
    }
    pub(super) fn netlist_navigation(&mut self, ui: &mut egui::Ui) {
        let Some(l) = self.circuit.netlist.clone() else {
            return;
        };
        ui.label(RichText::new(tr("УЧАСТКИ АЛУ")).size(10.0).color(muted()));
        egui::ComboBox::from_id_salt("sections")
            .selected_text(tr("Перейти к участку…"))
            .width(155.0)
            .show_ui(ui, |ui| {
                for s in &l.sections {
                    if ui.button(tr(&s.name)).clicked() {
                        self.focus_gate(s.anchor);
                        ui.close_menu();
                    }
                }
            });
        ui.add(
            egui::TextEdit::singleline(&mut self.find_gate)
                .hint_text(tr("Имя или ID элемента"))
                .desired_width(155.0),
        );
        if ui.button(tr("Найти элемент")).clicked() {
            let q = self.find_gate.trim().to_lowercase();
            let id = q
                .parse::<NodeId>()
                .ok()
                .filter(|id| self.circuit.node(*id).is_some())
                .or_else(|| {
                    self.circuit
                        .nodes
                        .iter()
                        .find(|n| n.label.to_lowercase() == q)
                        .map(|n| n.id)
                })
                .or_else(|| {
                    self.circuit
                        .nodes
                        .iter()
                        .find(|n| !q.is_empty() && n.label.to_lowercase().contains(&q))
                        .map(|n| n.id)
                });
            if let Some(id) = id {
                self.focus_gate(id);
            } else {
                self.message(tr("Элемент не найден"), true);
            }
        }
        if ui.button(tr("Вся схема")).clicked() {
            self.fit_next = true;
        }
        ui.label(
            RichText::new(if xor_112::i18n::is_english() {
                format!(
                    "{} elements\n{} wires",
                    self.circuit.nodes.len(),
                    self.circuit.wires.len()
                )
            } else {
                format!(
                    "{} элементов\n{} проводов",
                    self.circuit.nodes.len(),
                    self.circuit.wires.len()
                )
            })
            .size(11.0)
            .color(muted()),
        );
        ui.separator();
    }
    pub(super) fn netlist_controls(&mut self, ui: &mut egui::Ui, eval: &Evaluation) {
        let Some(l) = self.circuit.netlist.clone() else {
            return;
        };
        if let Some(id) = self.selected {
            ui.label(
                RichText::new(if xor_112::i18n::is_english() {
                    format!("ID: {id} · connections")
                } else {
                    format!("ID: {id} · соединения")
                })
                .size(11.0)
                .color(muted()),
            );
            let wires: Vec<_> = self
                .circuit
                .wires
                .iter()
                .filter(|w| w.to == id || w.from == id)
                .take(10)
                .copied()
                .collect();
            for w in wires {
                let other = if w.to == id { w.from } else { w.to };
                let label = self
                    .indexed_node(other)
                    .map(|n| n.label.as_str())
                    .unwrap_or("—");
                if ui
                    .small_button(format!(
                        "{} {label}",
                        if w.to == id { tr("Из") } else { tr("В") }
                    ))
                    .clicked()
                {
                    self.focus_gate(other);
                }
            }
            ui.separator();
        }
        egui::CollapsingHeader::new(tr("Входы и операция"))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(tr("Без знака · старший бит слева"))
                        .size(11.0)
                        .color(muted()),
                );
                for (name, draft) in [("A", &mut self.alu_a), ("B", &mut self.alu_b)] {
                    ui.label(name);
                    ui.add(
                        egui::TextEdit::multiline(draft)
                            .id_salt(name)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(192.0)
                            .desired_rows(l.bits.div_ceil(16))
                            .char_limit(95),
                    );
                }
                let valid_inputs =
                    l.a.iter()
                        .chain(&l.b)
                        .all(|&id| self.indexed_node(id).is_some_and(|n| n.kind == Kind::Input));
                let parsed = parse_bus(&self.alu_a, l.bits).zip(parse_bus(&self.alu_b, l.bits));
                if parsed.is_none() {
                    ui.colored_label(
                        error(),
                        if xor_112::i18n::is_english() {
                            format!("Enter 1 to {} bits: 0 and 1.", l.bits)
                        } else {
                            format!("Введите от 1 до {} бит: 0 и 1.", l.bits)
                        },
                    );
                }
                if !valid_inputs {
                    ui.colored_label(error(), tr("Часть входов A/B удалена."));
                }
                if ui
                    .add_enabled(
                        valid_inputs && parsed.is_some(),
                        egui::Button::new(tr("Применить входы")),
                    )
                    .clicked()
                {
                    let (a, b) = parsed.unwrap();
                    self.drive_netlist(a, b, None);
                }
                if ui.small_button(tr("Прочитать входы с холста")).clicked() {
                    self.sync_netlist_inputs();
                }
                let op = read_word(eval, &l.op);
                let can_select =
                    l.op.iter()
                        .all(|&id| self.indexed_node(id).is_some_and(|n| n.kind == Kind::Input));
                for (code, name) in [
                    tr("+  Сложение"),
                    tr("−  Вычитание"),
                    tr("×  Умножение"),
                    tr("÷  Деление"),
                    tr("√  Целый корень"),
                ]
                .iter()
                .enumerate()
                {
                    if ui
                        .add_enabled(
                            can_select,
                            egui::Button::new(format!("{code:03b}   {name}"))
                                .selected(op == Some(code as u64))
                                .corner_radius(9),
                        )
                        .clicked()
                    {
                        self.checkpoint();
                        for (i, id) in l.op.iter().enumerate() {
                            if let Some(n) = self.circuit.nodes.iter_mut().find(|n| n.id == *id) {
                                n.value = Bit::from_bool(code >> i & 1 != 0);
                            }
                        }
                    }
                }
                ui.label(
                    RichText::new(tr("Эти поля переключают обычные входы на холсте."))
                        .size(11.0)
                        .color(muted()),
                );
            });
        ui.separator();
        ui.label(
            RichText::new(tr("ВЫХОДЫ ВЕНТИЛЬНОЙ СХЕМЫ"))
                .size(10.0)
                .color(muted()),
        );
        for (name, ids) in [("R", &l.result), ("HI", &l.high), ("REM", &l.remainder)] {
            ui.label(RichText::new(name).strong());
            let text = ids
                .iter()
                .rev()
                .enumerate()
                .map(|(i, id)| {
                    format!(
                        "{}{}",
                        if i == 0 {
                            ""
                        } else if i % 16 == 0 {
                            "\n"
                        } else if i % 4 == 0 {
                            " "
                        } else {
                            ""
                        },
                        bit_text(eval.values.get(id).copied().flatten())
                    )
                })
                .collect::<String>();
            ui.label(RichText::new(text).monospace().size(12.0).color(accent()));
        }
        ui.separator();
        for (label, id) in ["CARRY", "BORROW", "OVERFLOW", "DIV0", "BAD_OP", "VALID"]
            .iter()
            .zip(&l.flags)
        {
            ui.label(
                RichText::new(format!(
                    "{label}  {}",
                    bit_text(eval.values.get(id).copied().flatten())
                ))
                .monospace()
                .size(12.0),
            );
        }
        ui.label(RichText::new(tr("DIV0 = 1: деление на ноль. VALID = 0: результат недействителен. REM — остаток деления или корня.")).size(11.0).color(muted()));
        ui.separator();
    }
}
fn read_word(eval: &Evaluation, ids: &[NodeId]) -> Option<u64> {
    ids.iter().enumerate().try_fold(0u64, |v, (i, id)| {
        Some(v | ((eval.values.get(id).copied().flatten()?.is_one() as u64) << i))
    })
}

fn parse_bus(text: &str, width: usize) -> Option<u64> {
    if text.chars().filter(|c| !c.is_whitespace()).count() > width {
        return None;
    }
    parse_word(text).map(|v| v.0)
}
fn format_bus(value: u64, width: usize) -> String {
    let bits = format!("{value:0width$b}");
    bits.as_bytes()
        .chunks(16)
        .map(|row| {
            row.chunks(4)
                .map(|b| std::str::from_utf8(b).unwrap())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
#[cfg(test)]
mod bus_tests {
    use super::*;
    #[test]
    fn operands_respect_the_bus_width() {
        assert_eq!(parse_bus(&"1".repeat(32), 32), Some(u32::MAX as u64));
        assert_eq!(parse_bus(&"1".repeat(33), 32), None);
        assert_eq!(parse_bus(&"0".repeat(33), 32), None);
        assert_eq!(parse_bus(&"1".repeat(64), 64), Some(u64::MAX));
        assert_eq!(parse_bus("1001 0011", 32), Some(147));
        assert_eq!(
            format_bus(9, 32)
                .chars()
                .filter(|c| !c.is_whitespace())
                .count(),
            32
        );
        assert_eq!(
            format_bus(9, 64)
                .chars()
                .filter(|c| !c.is_whitespace())
                .count(),
            64
        );
    }
}

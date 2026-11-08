use crate::i18n::tr;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

pub type NodeId = u64;
pub const MAX_NODES: usize = 250_000;
pub const MAX_WIRES: usize = 600_000;

/// Signal values are strictly binary, including in the file format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Bit {
    #[default]
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
}
impl Bit {
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::One
        } else {
            Self::Zero
        }
    }
    pub fn is_one(self) -> bool {
        self == Self::One
    }
    pub fn flipped(self) -> Self {
        Self::from_bool(!self.is_one())
    }
    pub fn text(self) -> &'static str {
        if self.is_one() {
            "1"
        } else {
            "0"
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Input,
    Output,
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,
    Buffer,
}
impl Kind {
    pub const ALL: [Self; 10] = [
        Self::Input,
        Self::Output,
        Self::And,
        Self::Or,
        Self::Not,
        Self::Nand,
        Self::Nor,
        Self::Xor,
        Self::Xnor,
        Self::Buffer,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Input => tr("Вход"),
            Self::Output => tr("Выход"),
            Self::And => "AND",
            Self::Or => "OR",
            Self::Not => "NOT",
            Self::Nand => "NAND",
            Self::Nor => "NOR",
            Self::Xor => "XOR",
            Self::Xnor => "XNOR",
            Self::Buffer => "BUF",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Input => tr("Переключатель 0 / 1"),
            Self::Output => tr("Индикатор результата"),
            Self::And => tr("И · оба входа равны 1"),
            Self::Or => tr("ИЛИ · хотя бы один вход равен 1"),
            Self::Not => tr("НЕ · инверсия входа"),
            Self::Nand => tr("И-НЕ · инверсия AND"),
            Self::Nor => tr("ИЛИ-НЕ · инверсия OR"),
            Self::Xor => tr("Исключающее ИЛИ · входы различны"),
            Self::Xnor => tr("Эквивалентность · входы совпадают"),
            Self::Buffer => tr("Повторитель · копирует вход"),
        }
    }
    pub fn inputs(self) -> usize {
        match self {
            Self::Input => 0,
            Self::Output | Self::Not | Self::Buffer => 1,
            _ => 2,
        }
    }
    pub fn has_output(self) -> bool {
        self != Self::Output
    }
    pub fn apply(self, a: Bit, b: Bit) -> Bit {
        let (a, b) = (a.is_one(), b.is_one());
        Bit::from_bool(match self {
            Self::And => a && b,
            Self::Or => a || b,
            Self::Not => !a,
            Self::Nand => !(a && b),
            Self::Nor => !(a || b),
            Self::Xor => a ^ b,
            Self::Xnor => a == b,
            Self::Input | Self::Output | Self::Buffer => a,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: NodeId,
    pub kind: Kind,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub value: Bit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wire {
    pub from: NodeId,
    pub to: NodeId,
    pub port: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circuit {
    pub version: u32,
    pub title: String,
    pub nodes: Vec<Node>,
    pub wires: Vec<Wire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alu64: Option<crate::alu64::Alu64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netlist: Option<crate::netlist::Layout>,
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    /// None is a diagnostic (incomplete wiring), never a third signal value.
    pub values: HashMap<NodeId, Option<Bit>>,
    pub missing: Vec<(NodeId, usize)>,
}

#[derive(Clone, Debug)]
pub struct TruthTable {
    pub inputs: Vec<(NodeId, String)>,
    pub outputs: Vec<(NodeId, String)>,
    pub rows: Vec<(Vec<Bit>, Vec<Bit>)>,
}

impl Default for Circuit {
    fn default() -> Self {
        Self {
            version: 1,
            title: tr("Новая схема").into(),
            nodes: vec![],
            wires: vec![],
            alu64: None,
            netlist: None,
        }
    }
}

impl Circuit {
    pub fn alu32_example() -> Self {
        crate::netlist::build(32)
    }
    pub fn alu64_example() -> Self {
        crate::netlist::build(64)
    }
    pub fn legacy_alu64_example() -> Self {
        Self {
            version: 2,
            title: tr("АЛУ · 64 бита").into(),
            alu64: Some(crate::alu64::Alu64::default()),
            ..Self::default()
        }
    }
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }
    pub fn add(&mut self, kind: Kind, x: f32, y: f32) -> Result<NodeId, String> {
        if self.alu64.is_some() {
            return Err(
                tr("Этот пример использует фиксированную функциональную схему АЛУ.").into(),
            );
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(tr("Достигнут предел элементов схемы.").into());
        }
        let id = self
            .nodes
            .iter()
            .map(|n| n.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(tr("Исчерпаны идентификаторы."))?;
        let label = match kind {
            Kind::Input => tr("Вход"),
            Kind::Output => tr("Выход"),
            _ => "",
        }
        .into();
        self.nodes.push(Node {
            id,
            kind,
            label,
            x,
            y,
            value: Bit::Zero,
        });
        Ok(id)
    }
    pub fn remove(&mut self, id: NodeId) {
        self.nodes.retain(|n| n.id != id);
        self.wires.retain(|w| w.from != id && w.to != id);
    }
    pub fn connect(&mut self, from: NodeId, to: NodeId, port: usize) -> Result<(), String> {
        let w = Wire { from, to, port };
        if self.wires.contains(&w) {
            return Ok(());
        }
        self.wires.push(w);
        if let Err(e) = self.validate() {
            self.wires.pop();
            return Err(e);
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version
            != if self.netlist.is_some() {
                3
            } else if self.alu64.is_some() {
                2
            } else {
                1
            }
        {
            return Err(tr("Эта версия формата схемы не поддерживается.").into());
        }
        if let Some(l) = &self.netlist {
            if self.alu64.is_some()
                || !(2..=64).contains(&l.bits)
                || [&l.a, &l.b, &l.result, &l.high, &l.remainder]
                    .iter()
                    .any(|v| v.len() != l.bits)
                || l.op.len() != 3
                || l.flags.len() != 6
                || l.sections.len() > 32
                || l.sections.iter().any(|s| s.name.chars().count() > 48)
            {
                return Err(tr("Некорректные подписи шин развёрнутой схемы.").into());
            }
            // References may become absent after a user deletes an element.
            // They describe navigation/monitors only, never circuit behavior.
        }
        if self.alu64.is_some() && (!self.nodes.is_empty() || !self.wires.is_empty()) {
            return Err(tr("Схема АЛУ не может содержать отдельные однобитные элементы.").into());
        }
        if self.title.chars().count() > 120 {
            return Err(tr("Слишком длинное название схемы.").into());
        }
        if self.nodes.len() > MAX_NODES || self.wires.len() > MAX_WIRES {
            return Err(tr("Схема превышает допустимый размер.").into());
        }
        let mut ids = HashSet::new();
        let mut ports = HashSet::new();
        for n in &self.nodes {
            if !ids.insert(n.id) {
                return Err(tr("Повторяющийся идентификатор элемента.").into());
            }
            if !n.x.is_finite()
                || !n.y.is_finite()
                || n.x.abs() > 100_000.0
                || n.y.abs() > 100_000.0
            {
                return Err(tr("Недопустимые координаты элемента.").into());
            }
            if n.label.chars().count() > 48 {
                return Err(tr("Подпись элемента слишком длинная.").into());
            }
        }
        let nodes: HashMap<_, _> = self.nodes.iter().map(|n| (n.id, n)).collect();
        for w in &self.wires {
            let a = nodes
                .get(&w.from)
                .ok_or(tr("Провод ссылается на отсутствующий источник."))?;
            let b = nodes
                .get(&w.to)
                .ok_or(tr("Провод ссылается на отсутствующий приёмник."))?;
            if !a.kind.has_output() || w.port >= b.kind.inputs() {
                return Err(tr("Недопустимое направление или контакт провода.").into());
            }
            if !ports.insert((w.to, w.port)) {
                return Err(tr(
                    "Этот вход уже занят. Сначала удалите его провод правой кнопкой мыши.",
                )
                .into());
            }
        }
        self.order()?;
        Ok(())
    }
    fn order(&self) -> Result<Vec<NodeId>, String> {
        let mut degree: HashMap<NodeId, usize> = self.nodes.iter().map(|n| (n.id, 0)).collect();
        let mut outgoing: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for w in &self.wires {
            *degree.get_mut(&w.to).ok_or(tr("Не найден приёмник."))? += 1;
            outgoing.entry(w.from).or_default().push(w.to);
        }
        let mut queue: VecDeque<_> = self
            .nodes
            .iter()
            .filter(|n| degree[&n.id] == 0)
            .map(|n| n.id)
            .collect();
        let mut order = Vec::new();
        while let Some(id) = queue.pop_front() {
            order.push(id);
            if let Some(next) = outgoing.get(&id) {
                for to in next {
                    let d = degree.get_mut(to).ok_or(tr("Не найден элемент."))?;
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(*to);
                    }
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err(tr("Обратная связь создаёт цикл. В этой версии поддерживаются комбинационные схемы без памяти.").into());
        }
        Ok(order)
    }
    pub fn evaluate(&self) -> Result<Evaluation, String> {
        self.validate()?;
        let nodes: HashMap<_, _> = self.nodes.iter().map(|n| (n.id, n)).collect();
        let wires: HashMap<_, _> = self
            .wires
            .iter()
            .map(|w| ((w.to, w.port), w.from))
            .collect();
        let mut values: HashMap<NodeId, Option<Bit>> = HashMap::new();
        let mut missing = vec![];
        for id in self.order()? {
            let n = nodes[&id];
            if n.kind == Kind::Input {
                values.insert(id, Some(n.value));
                continue;
            }
            let mut args = vec![];
            for port in 0..n.kind.inputs() {
                match wires.get(&(id, port)) {
                    Some(from) => args.push(values.get(from).copied().flatten()),
                    None => {
                        missing.push((id, port));
                        args.push(None);
                    }
                }
            }
            let value = if args.iter().any(Option::is_none) {
                None
            } else {
                Some(n.kind.apply(
                    args[0].unwrap(),
                    args.get(1).copied().flatten().unwrap_or(Bit::Zero),
                ))
            };
            values.insert(id, value);
        }
        Ok(Evaluation { values, missing })
    }
    pub fn truth_table(&self) -> Result<TruthTable, String> {
        if self.alu64.is_some() {
            return Err(tr("Полная таблица для 128 входных битов слишком велика. Проверяйте АЛУ изменением входов и выбором операции.").into());
        }
        self.validate()?;
        let inputs: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| n.kind == Kind::Input)
            .map(|n| (n.id, n.label.clone()))
            .collect();
        let outputs: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| n.kind == Kind::Output)
            .map(|n| (n.id, n.label.clone()))
            .collect();
        if inputs.len() > 8 {
            return Err(tr("Таблица истинности доступна для схем с числом входов до 8.").into());
        }
        if outputs.is_empty() {
            return Err(tr("Добавьте хотя бы один элемент «Выход».").into());
        }
        if !self.evaluate()?.missing.is_empty() {
            return Err(tr("Подключите все входные контакты схемы.").into());
        }
        let mut copy = self.clone();
        let mut rows = vec![];
        for mask in 0..(1usize << inputs.len()) {
            let bits: Vec<_> = (0..inputs.len())
                .map(|i| Bit::from_bool(mask & (1 << (inputs.len() - 1 - i)) != 0))
                .collect();
            for ((id, _), bit) in inputs.iter().zip(&bits) {
                copy.nodes.iter_mut().find(|n| n.id == *id).unwrap().value = *bit;
            }
            let evaluation = copy.evaluate()?;
            let out: Result<Vec<_>, _> = outputs
                .iter()
                .map(|(id, _)| evaluation.values[id].ok_or(tr("Не удалось вычислить выход.")))
                .collect();
            rows.push((bits, out?));
        }
        Ok(TruthTable {
            inputs,
            outputs,
            rows,
        })
    }
    pub fn half_adder() -> Self {
        let mut c = Self {
            title: tr("Полусумматор").into(),
            ..Self::default()
        };
        let a = c.add(Kind::Input, 80.0, 100.0).unwrap();
        let b = c.add(Kind::Input, 80.0, 300.0).unwrap();
        let xor = c.add(Kind::Xor, 340.0, 100.0).unwrap();
        let and = c.add(Kind::And, 340.0, 300.0).unwrap();
        let sum = c.add(Kind::Output, 600.0, 100.0).unwrap();
        let carry = c.add(Kind::Output, 600.0, 300.0).unwrap();
        for (id, label) in [
            (a, "A"),
            (b, "B"),
            (sum, tr("Сумма")),
            (carry, tr("Перенос")),
        ] {
            c.nodes.iter_mut().find(|n| n.id == id).unwrap().label = label.into();
        }
        c.nodes[0].value = Bit::One;
        for (from, to, port) in [
            (a, xor, 0),
            (b, xor, 1),
            (a, and, 0),
            (b, and, 1),
            (xor, sum, 0),
            (and, carry, 0),
        ] {
            c.connect(from, to, port).unwrap();
        }
        c
    }
    pub fn full_adder() -> Self {
        let mut c = Self {
            title: tr("Полный сумматор").into(),
            ..Self::default()
        };
        for (kind, x, y, label) in [
            (Kind::Input, 40.0, 80.0, "A"),
            (Kind::Input, 40.0, 240.0, "B"),
            (Kind::Input, 40.0, 400.0, "Cin"),
            (Kind::Xor, 260.0, 80.0, ""),
            (Kind::And, 260.0, 260.0, ""),
            (Kind::Xor, 480.0, 80.0, ""),
            (Kind::And, 480.0, 300.0, ""),
            (Kind::Or, 700.0, 300.0, ""),
            (Kind::Output, 900.0, 80.0, tr("Сумма")),
            (Kind::Output, 900.0, 300.0, "Cout"),
        ] {
            let id = c.add(kind, x, y).unwrap();
            c.nodes.iter_mut().find(|n| n.id == id).unwrap().label = label.into();
        }
        for (from, to, port) in [
            (1, 4, 0),
            (2, 4, 1),
            (1, 5, 0),
            (2, 5, 1),
            (4, 6, 0),
            (3, 6, 1),
            (4, 7, 0),
            (3, 7, 1),
            (5, 8, 0),
            (7, 8, 1),
            (6, 9, 0),
            (8, 10, 0),
        ] {
            c.connect(from, to, port).unwrap();
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_gate_truth_table() {
        let rows = [(false, false), (false, true), (true, false), (true, true)];
        for (kind, expected) in [
            (Kind::And, [0, 0, 0, 1]),
            (Kind::Or, [0, 1, 1, 1]),
            (Kind::Xor, [0, 1, 1, 0]),
            (Kind::Nand, [1, 1, 1, 0]),
            (Kind::Nor, [1, 0, 0, 0]),
            (Kind::Xnor, [1, 0, 0, 1]),
            (Kind::Not, [1, 1, 0, 0]),
            (Kind::Buffer, [0, 0, 1, 1]),
        ] {
            for ((a, b), e) in rows.into_iter().zip(expected) {
                assert_eq!(
                    kind.apply(Bit::from_bool(a), Bit::from_bool(b)).is_one(),
                    e == 1
                );
            }
        }
    }
    #[test]
    fn half_adder_all_inputs_and_fanout() {
        let t = Circuit::half_adder().truth_table().unwrap();
        assert_eq!(
            t.rows
                .iter()
                .map(|(_, o)| o.iter().map(|b| b.text()).collect::<String>())
                .collect::<Vec<_>>(),
            ["00", "10", "10", "01"]
        );
    }
    #[test]
    fn full_adder_all_inputs() {
        for (i, o) in Circuit::full_adder().truth_table().unwrap().rows {
            let total = i.iter().filter(|b| b.is_one()).count();
            assert_eq!(o[0].is_one(), total % 2 == 1);
            assert_eq!(o[1].is_one(), total >= 2);
        }
    }
    #[test]
    fn reject_cycle_and_keep_original() {
        let mut c = Circuit::default();
        c.add(Kind::Not, 0.0, 0.0).unwrap();
        c.add(Kind::Not, 1.0, 0.0).unwrap();
        c.connect(1, 2, 0).unwrap();
        let original = c.clone();
        assert!(c.connect(2, 1, 0).is_err());
        assert_eq!(c, original);
        assert!(c.connect(1, 1, 0).is_err());
    }
    #[test]
    fn reject_multiple_drivers_and_bad_ports() {
        let mut c = Circuit::half_adder();
        let original = c.clone();
        assert!(c.connect(2, 3, 0).is_err());
        assert_eq!(c, original);
        assert!(c.connect(1, 3, 2).is_err());
        assert!(c.connect(5, 3, 0).is_err());
        assert!(c.connect(99, 3, 0).is_err());
    }
    #[test]
    fn missing_inputs_are_diagnostics_and_deletion_cleans_wires() {
        let mut c = Circuit::half_adder();
        c.remove(1);
        assert!(c.wires.iter().all(|w| w.from != 1 && w.to != 1));
        let e = c.evaluate().unwrap();
        assert_eq!(e.missing.len(), 2);
        assert_eq!(e.values[&5], None);
        assert!(c.truth_table().is_err());
    }
    #[test]
    fn evaluation_does_not_depend_on_node_order() {
        let mut c = Circuit::full_adder();
        let before = c.evaluate().unwrap().values;
        c.nodes.reverse();
        assert_eq!(c.evaluate().unwrap().values, before);
    }
    #[test]
    fn table_limit_and_duplicate_ids() {
        let mut c = Circuit::half_adder();
        for _ in 0..7 {
            c.add(Kind::Input, 0.0, 0.0).unwrap();
        }
        assert!(c.truth_table().is_err());
        c.nodes[1].id = c.nodes[0].id;
        assert!(c.validate().is_err());
    }
}

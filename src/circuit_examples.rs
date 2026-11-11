//! Editable textbook circuits. Multi-input gates are expanded into binary gates
//! to keep existing .binary documents and the elementary gate library compatible.
use crate::{
    i18n::tr,
    model::{Circuit, Kind, NodeId, Wire, MAX_NODES, MAX_WIRES},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    HalfAdder,
    FullAdder,
    Majority,
    Mux8,
}

impl Template {
    pub const ALL: [Self; 4] = [Self::HalfAdder, Self::FullAdder, Self::Majority, Self::Mux8];
    pub fn name(self) -> &'static str {
        match self {
            Self::HalfAdder => tr("Полусумматор"),
            Self::FullAdder => tr("Полный сумматор"),
            Self::Majority => tr("Большинство · 3 входа"),
            Self::Mux8 => tr("Мультиплексор · 8:1"),
        }
    }
    pub fn build(self) -> Circuit {
        match self {
            Self::HalfAdder => Circuit::half_adder(),
            Self::FullAdder => Circuit::full_adder(),
            Self::Majority => Circuit::majority(),
            Self::Mux8 => Circuit::mux8(),
        }
    }
}

fn node(c: &mut Circuit, kind: Kind, x: f32, y: f32, label: &str) -> NodeId {
    let id = c.add(kind, x, y).expect("small built-in circuit");
    c.nodes.last_mut().unwrap().label = label.into();
    id
}
fn gate(c: &mut Circuit, kind: Kind, x: f32, y: f32, label: &str, inputs: &[NodeId]) -> NodeId {
    let id = node(c, kind, x, y, label);
    for (port, &from) in inputs.iter().enumerate() {
        c.wires.push(Wire { from, to: id, port });
    }
    id
}

impl Circuit {
    /// Tanenbaum, fig. 3.3: !ABC + A!BC + AB!C + ABC.
    pub fn majority() -> Self {
        let mut c = Self {
            title: Template::Majority.name().into(),
            ..Self::default()
        };
        let a = node(&mut c, Kind::Input, 0., 80., "A");
        let b = node(&mut c, Kind::Input, 0., 280., "B");
        let cc = node(&mut c, Kind::Input, 0., 480., "C");
        let na = gate(&mut c, Kind::Not, 200., 80., "¬A", &[a]);
        let nb = gate(&mut c, Kind::Not, 200., 280., "¬B", &[b]);
        let nc = gate(&mut c, Kind::Not, 200., 480., "¬C", &[cc]);
        let mut terms = Vec::new();
        for (row, (inputs, label)) in [
            ([na, b, cc], "¬A·B·C"),
            ([a, nb, cc], "A·¬B·C"),
            ([a, b, nc], "A·B·¬C"),
            ([a, b, cc], "A·B·C"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = row as f32 * 160.;
            let ab = gate(&mut c, Kind::And, 420., y, "", &inputs[..2]);
            terms.push(gate(&mut c, Kind::And, 620., y, label, &[ab, inputs[2]]));
        }
        let first = gate(&mut c, Kind::Or, 820., 80., "", &terms[..2]);
        let last = gate(&mut c, Kind::Or, 820., 400., "", &terms[2..]);
        let combined = gate(&mut c, Kind::Or, 1020., 240., "", &[first, last]);
        gate(&mut c, Kind::Output, 1220., 240., "M", &[combined]);
        c
    }

    /// Tanenbaum, fig. 3.10. ABC is the binary address: 000 selects D0,
    /// 001 selects D1, ..., 111 selects D7 (A is the most significant bit).
    pub fn mux8() -> Self {
        let mut c = Self {
            title: Template::Mux8.name().into(),
            ..Self::default()
        };
        let data: Vec<_> = (0..8)
            .map(|i| node(&mut c, Kind::Input, 0., i as f32 * 160., &format!("D{i}")))
            .collect();
        let mut select = Vec::new();
        let mut inverse = Vec::new();
        for (i, label) in ["A", "B", "C"].into_iter().enumerate() {
            let x = i as f32 * 240.;
            let id = node(&mut c, Kind::Input, x, 1480., label);
            select.push(id);
            inverse.push(gate(
                &mut c,
                Kind::Not,
                x,
                1320.,
                &format!("¬{label}"),
                &[id],
            ));
        }
        let mut terms = Vec::new();
        for (i, &d) in data.iter().enumerate() {
            let y = i as f32 * 160.;
            let address: Vec<_> = (0..3)
                .map(|bit| {
                    if i & (1 << (2 - bit)) != 0 {
                        select[bit]
                    } else {
                        inverse[bit]
                    }
                })
                .collect();
            let ab = gate(&mut c, Kind::And, 280., y, "", &address[..2]);
            let abc = gate(
                &mut c,
                Kind::And,
                520.,
                y,
                &format!("{i:03b}"),
                &[ab, address[2]],
            );
            terms.push(gate(
                &mut c,
                Kind::And,
                760.,
                y,
                &format!("D{i}·{i:03b}"),
                &[d, abc],
            ));
        }
        let mut level = terms;
        for depth in 0..3 {
            let spacing = 160. * 2_f32.powi(depth + 1);
            level = level
                .chunks_exact(2)
                .enumerate()
                .map(|(i, pair)| {
                    gate(
                        &mut c,
                        Kind::Or,
                        1000. + depth as f32 * 220.,
                        i as f32 * spacing + (spacing - 160.) / 2.,
                        "",
                        pair,
                    )
                })
                .collect();
        }
        gate(&mut c, Kind::Output, 1660., 560., "F", &level);
        c
    }

    /// Append an independent editable copy; failures leave the document intact.
    /// x/y locate the upper left node of the normalized template layout.
    pub fn insert_template(
        &mut self,
        template: Template,
        x: f32,
        y: f32,
    ) -> Result<Vec<NodeId>, String> {
        if self.alu64.is_some() {
            return Err(
                tr("Этот пример использует фиксированную функциональную схему АЛУ.").into(),
            );
        }
        let source = template.build();
        let (old_nodes, old_wires) = (self.nodes.len(), self.wires.len());
        if old_nodes + source.nodes.len() > MAX_NODES || old_wires + source.wires.len() > MAX_WIRES
        {
            return Err(tr("Схема превышает допустимый размер.").into());
        }
        let base = self.nodes.iter().map(|n| n.id).max().unwrap_or(0);
        base.checked_add(source.nodes.len() as u64)
            .ok_or(tr("Исчерпаны идентификаторы."))?;
        let min_x = source
            .nodes
            .iter()
            .map(|n| n.x)
            .fold(f32::INFINITY, f32::min);
        let min_y = source
            .nodes
            .iter()
            .map(|n| n.y)
            .fold(f32::INFINITY, f32::min);
        let mut ids = HashMap::new();
        for (i, mut n) in source.nodes.into_iter().enumerate() {
            let new_id = base + i as u64 + 1;
            ids.insert(n.id, new_id);
            n.id = new_id;
            n.x += x - min_x;
            n.y += y - min_y;
            self.nodes.push(n);
        }
        self.wires.extend(source.wires.into_iter().map(|w| Wire {
            from: ids[&w.from],
            to: ids[&w.to],
            port: w.port,
        }));
        if let Err(e) = self.validate() {
            self.nodes.truncate(old_nodes);
            self.wires.truncate(old_wires);
            return Err(e);
        }
        Ok(self.nodes[old_nodes..].iter().map(|n| n.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Bit;

    #[test]
    fn majority_all_eight_inputs() {
        let c = Circuit::majority();
        c.validate().unwrap();
        for (inputs, outputs) in c.truth_table().unwrap().rows {
            assert_eq!(
                outputs,
                vec![Bit::from_bool(
                    inputs.iter().filter(|b| b.is_one()).count() >= 2
                )]
            );
        }
    }
    #[test]
    fn mux_all_data_words_and_addresses() {
        let mut c = Circuit::mux8();
        c.validate().unwrap();
        let output = c.nodes.iter().find(|n| n.kind == Kind::Output).unwrap().id;
        for word in 0..256 {
            for address in 0..8 {
                for n in c.nodes.iter_mut().filter(|n| n.kind == Kind::Input) {
                    n.value = Bit::from_bool(match n.label.as_str() {
                        "A" => address & 4 != 0,
                        "B" => address & 2 != 0,
                        "C" => address & 1 != 0,
                        label => word & (1 << label[1..].parse::<u32>().unwrap()) != 0,
                    });
                }
                let result = c.evaluate().unwrap();
                assert_eq!(
                    result.values[&output],
                    Some(Bit::from_bool(word & (1 << address) != 0)),
                    "word {word}, address {address}"
                );
            }
        }
    }
    #[test]
    fn repeated_template_insertions_preserve_document_and_roundtrip() {
        let mut c = Circuit::half_adder();
        let original = c.clone();
        for template in Template::ALL.into_iter().cycle().take(8) {
            c.insert_template(template, 2000., 2000.).unwrap();
        }
        assert_eq!(c.title, original.title);
        assert_eq!(c.nodes[..original.nodes.len()], original.nodes);
        assert_eq!(c.wires[..original.wires.len()], original.wires);
        c.validate().unwrap();
        assert!(c.evaluate().unwrap().missing.is_empty());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("templates.binary");
        crate::storage::save(&path, &c).unwrap();
        assert_eq!(crate::storage::load(&path).unwrap(), c);
        for (x, y) in [(f32::NAN, 0.), (100_000., 0.), (0., -100_001.)] {
            let before = c.clone();
            assert!(c.insert_template(Template::Mux8, x, y).is_err());
            assert_eq!(c, before);
        }
        c.nodes[0].id = u64::MAX;
        let before = c.clone();
        assert!(c.insert_template(Template::HalfAdder, 0., 0.).is_err());
        assert_eq!(c, before);
    }
}

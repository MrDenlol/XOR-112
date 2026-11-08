//! A generator, not an evaluator: every result is produced by ordinary editable gates.
use crate::model::{Bit, Circuit, Kind, Node, NodeId, Wire};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub name: String,
    pub anchor: NodeId,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub bits: usize,
    pub a: Vec<NodeId>,
    pub b: Vec<NodeId>,
    pub op: Vec<NodeId>,
    pub result: Vec<NodeId>,
    pub high: Vec<NodeId>,
    pub remainder: Vec<NodeId>,
    pub flags: Vec<NodeId>,
    pub sections: Vec<Section>,
}
struct Builder {
    c: Circuit,
    zero: NodeId,
    one: NodeId,
    x: f32,
    y: f32,
    col: usize,
    count: usize,
    prefix: String,
    cache: HashMap<(Kind, NodeId, NodeId), NodeId>,
}
impl Builder {
    fn new() -> Self {
        Self {
            c: Circuit::default(),
            zero: 0,
            one: 0,
            x: 0.,
            y: 0.,
            col: 16,
            count: 0,
            prefix: String::new(),
            cache: HashMap::new(),
        }
    }
    fn section(&mut self, name: &str, x: f32, y: f32, col: usize, sections: &mut Vec<Section>) {
        self.x = x;
        self.y = y;
        self.col = col;
        self.count = 0;
        self.prefix = name.into();
        sections.push(Section {
            name: name.into(),
            anchor: self.c.nodes.len() as u64 + 1,
        });
    }
    fn raw(&mut self, kind: Kind, args: &[NodeId], label: String) -> NodeId {
        let id = self.c.nodes.len() as u64 + 1;
        let x = self.x + (self.count % self.col) as f32 * 180.;
        let y = self.y + (self.count / self.col) as f32 * 140.;
        self.count += 1;
        self.c.nodes.push(Node {
            id,
            kind,
            label,
            x,
            y,
            value: Bit::Zero,
        });
        for (port, &from) in args.iter().enumerate() {
            self.c.wires.push(Wire { from, to: id, port });
        }
        id
    }
    fn gate(&mut self, k: Kind, a: NodeId, b: NodeId) -> NodeId {
        if self.zero != 0 {
            match k {
                Kind::And => {
                    if a == self.zero || b == self.zero {
                        return self.zero;
                    }
                    if a == self.one {
                        return b;
                    }
                    if b == self.one || a == b {
                        return a;
                    }
                }
                Kind::Or => {
                    if a == self.one || b == self.one {
                        return self.one;
                    }
                    if a == self.zero {
                        return b;
                    }
                    if b == self.zero || a == b {
                        return a;
                    }
                }
                Kind::Xor => {
                    if a == b {
                        return self.zero;
                    }
                    if a == self.zero {
                        return b;
                    }
                    if b == self.zero {
                        return a;
                    }
                }
                Kind::Not => {
                    if a == self.zero {
                        return self.one;
                    }
                    if a == self.one {
                        return self.zero;
                    }
                }
                _ => {}
            }
        }
        let (a, b) = if k != Kind::Not && b < a {
            (b, a)
        } else {
            (a, b)
        };
        let key = (k, a, if k == Kind::Not { 0 } else { b });
        if let Some(&id) = self.cache.get(&key) {
            return id;
        }
        let label = format!("{}/{}", self.prefix, self.count);
        let id = if k == Kind::Not {
            self.raw(k, &[a], label)
        } else {
            self.raw(k, &[a, b], label)
        };
        self.cache.insert(key, id);
        id
    }
    fn and(&mut self, a: NodeId, b: NodeId) -> NodeId {
        self.gate(Kind::And, a, b)
    }
    fn or(&mut self, a: NodeId, b: NodeId) -> NodeId {
        self.gate(Kind::Or, a, b)
    }
    fn xor(&mut self, a: NodeId, b: NodeId) -> NodeId {
        self.gate(Kind::Xor, a, b)
    }
    fn not(&mut self, a: NodeId) -> NodeId {
        self.gate(Kind::Not, a, 0)
    }
    fn mux(&mut self, select: NodeId, no: NodeId, yes: NodeId) -> NodeId {
        if no == yes {
            return no;
        }
        let delta = self.xor(no, yes);
        let chosen = self.and(select, delta);
        self.xor(no, chosen)
    }
    fn add(&mut self, a: &[NodeId], b: &[NodeId], mut carry: NodeId) -> (Vec<NodeId>, NodeId) {
        let mut result = Vec::new();
        for (&a, &b) in a.iter().zip(b) {
            let p = self.xor(a, b);
            result.push(self.xor(p, carry));
            let g = self.and(a, b);
            let h = self.and(p, carry);
            carry = self.or(g, h);
        }
        (result, carry)
    }
    fn sub(&mut self, a: &[NodeId], b: &[NodeId]) -> (Vec<NodeId>, NodeId) {
        let inverted: Vec<_> = b.iter().map(|&b| self.not(b)).collect();
        self.add(a, &inverted, self.one)
    }
    fn any(&mut self, bits: &[NodeId]) -> NodeId {
        bits.iter().fold(self.zero, |v, &b| self.or(v, b))
    }
    fn select_words(
        &mut self,
        enable: &[NodeId],
        words: &[Vec<NodeId>],
        width: usize,
    ) -> Vec<NodeId> {
        (0..width)
            .map(|i| {
                let mut result = self.zero;
                for (&e, w) in enable.iter().zip(words) {
                    let term = self.and(e, w[i]);
                    result = self.or(result, term);
                }
                result
            })
            .collect()
    }
    fn outputs(&mut self, prefix: &str, bits: &[NodeId]) -> Vec<NodeId> {
        bits.iter()
            .enumerate()
            .map(|(i, &b)| self.raw(Kind::Output, &[b], format!("{prefix}[{i:02}]")))
            .collect()
    }
}

pub fn build(width: usize) -> Circuit {
    assert!((2..=64).contains(&width));
    let n = width;
    let mut g = Builder::new();
    let mut sections = Vec::new();
    g.section("Входы", 0., 0., 4, &mut sections);
    let a: Vec<_> = (0..n)
        .map(|i| g.raw(Kind::Input, &[], format!("A[{i:02}]")))
        .collect();
    let b: Vec<_> = (0..n)
        .map(|i| g.raw(Kind::Input, &[], format!("B[{i:02}]")))
        .collect();
    let op: Vec<_> = (0..3)
        .map(|i| g.raw(Kind::Input, &[], format!("OP[{i}]")))
        .collect();
    for (bits, value) in [(&a, 9u64), (&b, 3u64)] {
        for (i, &id) in bits.iter().enumerate() {
            g.c.nodes[id as usize - 1].value = Bit::from_bool(value >> i & 1 != 0);
        }
    }
    // Zero and one themselves are real gates, not evaluator constants.
    g.zero = g.raw(Kind::Xor, &[a[0], a[0]], "CONST/0".into());
    g.one = g.raw(Kind::Not, &[g.zero], "CONST/1".into());
    let zero = vec![g.zero; n];
    g.section("ADD", 3000., 0., 16, &mut sections);
    let (sum, carry) = g.add(&a, &b, g.zero);
    g.section("SUB", 6500., 0., 16, &mut sections);
    let (difference, no_borrow) = g.sub(&a, &b);
    let borrow = g.not(no_borrow);
    g.section("MUL", 0., 10000., 128, &mut sections);
    let mut product = vec![g.zero; n * 2];
    for (shift, &bit) in b.iter().enumerate() {
        let mut partial = vec![g.zero; n * 2];
        for (i, &ai) in a.iter().enumerate() {
            partial[i + shift] = g.and(ai, bit);
        }
        product = g.add(&product, &partial, g.zero).0;
    }
    let multiply_overflow = g.any(&product[n..]);
    g.section("DIV", 26000., 10000., 128, &mut sections);
    let mut divisor = b.clone();
    divisor.push(g.zero);
    let mut rem = vec![g.zero; n + 1];
    let mut quotient = vec![g.zero; n];
    for bit in (0..n).rev() {
        rem.rotate_right(1);
        rem[0] = a[bit];
        let (diff, ge) = g.sub(&rem, &divisor);
        rem = rem
            .into_iter()
            .zip(diff)
            .map(|(old, new)| g.mux(ge, old, new))
            .collect();
        quotient[bit] = ge;
    }
    let nonzero_b = g.any(&b);
    let divide_zero = g.not(nonzero_b);
    let quotient: Vec<_> = quotient.into_iter().map(|x| g.and(nonzero_b, x)).collect();
    let division_rem: Vec<_> = rem[..n].iter().map(|&x| g.and(nonzero_b, x)).collect();
    g.section("SQRT", 52000., 10000., 96, &mut sections);
    let mut root = vec![g.zero; n];
    let mut sr = vec![g.zero; n + 2];
    for pair in (0..n.div_ceil(2)).rev() {
        sr.rotate_right(2);
        sr[0] = a[pair * 2];
        sr[1] = a.get(pair * 2 + 1).copied().unwrap_or(g.zero);
        let mut trial = vec![g.zero; n + 2];
        trial[0] = g.one;
        trial[2..].copy_from_slice(&root);
        let (diff, ge) = g.sub(&sr, &trial);
        sr = sr
            .into_iter()
            .zip(diff)
            .map(|(old, new)| g.mux(ge, old, new))
            .collect();
        root.rotate_right(1);
        root[0] = ge;
    }
    g.section("SELECT", 11000., 0., 24, &mut sections);
    let enables: Vec<_> = (0..5)
        .map(|code| {
            let mut e = g.one;
            for (i, &bit) in op.iter().enumerate() {
                let term = if code >> i & 1 != 0 { bit } else { g.not(bit) };
                e = g.and(e, term);
            }
            e
        })
        .collect();
    let valid_op = g.any(&enables);
    let invalid_op = g.not(valid_op);
    let result = g.select_words(
        &enables,
        &[sum, difference, product[..n].to_vec(), quotient, root],
        n,
    );
    let mut carry_word = zero.clone();
    carry_word[0] = carry;
    let high = g.select_words(
        &enables,
        &[
            carry_word,
            zero.clone(),
            product[n..].to_vec(),
            zero.clone(),
            zero.clone(),
        ],
        n,
    );
    let remainder = g.select_words(
        &enables,
        &[
            zero.clone(),
            zero.clone(),
            zero.clone(),
            division_rem,
            sr[..n].to_vec(),
        ],
        n,
    );
    let carry_flag = g.and(enables[0], carry);
    let borrow_flag = g.and(enables[1], borrow);
    let mul_flag = g.and(enables[2], multiply_overflow);
    let addsub = g.or(carry_flag, borrow_flag);
    let overflow = g.or(addsub, mul_flag);
    let div_error = g.and(enables[3], divide_zero);
    let error = g.or(invalid_op, div_error);
    let valid = g.not(error);
    g.section("Выходы", 17000., 0., 8, &mut sections);
    let result = g.outputs("R", &result);
    let high = g.outputs("HI", &high);
    let remainder = g.outputs("REM", &remainder);
    let flags: Vec<_> = [
        ("CARRY", carry_flag),
        ("BORROW", borrow_flag),
        ("OVERFLOW", overflow),
        ("DIV0", div_error),
        ("BAD_OP", invalid_op),
        ("VALID", valid),
    ]
    .into_iter()
    .map(|(label, source)| g.raw(Kind::Output, &[source], label.into()))
    .collect();
    g.c.version = 3;
    g.c.title = if crate::i18n::is_english() {
        format!("ALU · {n} bits · gates")
    } else {
        format!("АЛУ · {n} бита · вентили")
    };
    g.c.netlist = Some(Layout {
        bits: n,
        a,
        b,
        op,
        result,
        high,
        remainder,
        flags,
        sections,
    });
    g.c.validate().expect("generated graph must be valid");
    g.c
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drive(c: &mut Circuit, a: u64, b: u64, op: u64) {
        let l = c.netlist.clone().unwrap();
        for (ids, value) in [(&l.a, a), (&l.b, b), (&l.op, op)] {
            for (i, id) in ids.iter().enumerate() {
                c.nodes[*id as usize - 1].value = Bit::from_bool(value >> i & 1 != 0);
            }
        }
    }
    fn word(e: &crate::model::Evaluation, ids: &[NodeId]) -> u64 {
        ids.iter().enumerate().fold(0, |v, (i, id)| {
            v | ((e.values[id].unwrap().is_one() as u64) << i)
        })
    }
    fn check(c: &mut Circuit, a: u64, b: u64, op: u64) {
        drive(c, a, b, op);
        let e = c.evaluate().unwrap();
        let l = c.netlist.as_ref().unwrap();
        let mask = if l.bits == 64 {
            u64::MAX
        } else {
            (1 << l.bits) - 1
        };
        let r = word(&e, &l.result);
        let hi = word(&e, &l.high);
        let rem = word(&e, &l.remainder);
        let carry = op == 0 && (a as u128 + b as u128) > mask as u128;
        let borrow = op == 1 && a < b;
        let overflow = carry || borrow || (op == 2 && (a as u128 * b as u128) > mask as u128);
        let div0 = op == 3 && b == 0;
        let bad = op > 4;
        for (&id, expected) in
            l.flags
                .iter()
                .zip([carry, borrow, overflow, div0, bad, !div0 && !bad])
        {
            assert_eq!(
                e.values[&id],
                Some(Bit::from_bool(expected)),
                "flag {id}, A={a}, B={b}, OP={op}"
            );
        }
        match op {
            0 => {
                let x = a as u128 + b as u128;
                assert_eq!(r, x as u64 & mask);
                assert_eq!(hi, (x >> l.bits) as u64);
            }
            1 => assert_eq!(r, a.wrapping_sub(b) & mask),
            2 => {
                let x = a as u128 * b as u128;
                assert_eq!(r, x as u64 & mask);
                assert_eq!(hi, (x >> l.bits) as u64);
            }
            3 => {
                if b != 0 {
                    assert_eq!(r, a / b);
                    assert_eq!(rem, a % b);
                } else {
                    assert_eq!(r, 0);
                    assert!(e.values[&l.flags[3]].unwrap().is_one());
                }
            }
            4 => {
                let root = r as u128;
                assert!(root * root <= a as u128, "sqrt({a})={r}");
                assert!((root + 1) * (root + 1) > a as u128);
                assert_eq!(root * root + rem as u128, a as u128);
            }
            _ => {
                assert_eq!(r, 0);
                assert!(e.values[&l.flags[4]].unwrap().is_one());
            }
        }
    }
    #[test]
    fn exhaustive_small_gate_alu() {
        let mut c = build(4);
        for a in 0..16 {
            for b in 0..16 {
                for op in 0..8 {
                    check(&mut c, a, b, op);
                }
            }
        }
    }
    #[test]
    fn alu32_arithmetic_editing_and_save_load() {
        let mut c = Circuit::alu32_example();
        let max = u32::MAX as u64;
        for (a, b) in [
            (0, 0),
            (9, 3),
            (max, 1),
            (max, max),
            (1 << 31, 7),
            (max, 97),
            (65535 * 65535, 3),
        ] {
            for op in 0..8 {
                check(&mut c, a, b, op);
            }
        }
        let mut state = 0x129e8367u32;
        for _ in 0..12 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let a = state as u64;
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let b = state as u64;
            for op in 0..5 {
                check(&mut c, a, b, op);
            }
        }
        let l = c.netlist.clone().unwrap();
        assert_eq!(l.bits, 32);
        assert!(c.alu64.is_none());
        drive(&mut c, 9, 3, 0);
        let out = l.result[0];
        c.wires.retain(|w| w.to != out);
        c.connect(l.a[0], out, 0).unwrap();
        assert_eq!(word(&c.evaluate().unwrap(), &l.result), 13);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("alu32.binary");
        crate::storage::save(&path, &c).unwrap();
        assert_eq!(crate::storage::load(&path).unwrap(), c);
        eprintln!("ALU32: {} elements, {} wires", c.nodes.len(), c.wires.len());
    }
    #[test]
    fn full_width_gate_alu_and_editability() {
        let mut c = build(64);
        assert!(c.alu64.is_none());
        assert!(c.nodes.iter().all(|n| Kind::ALL.contains(&n.kind)));
        for (a, b) in [
            (0, 0),
            (9, 3),
            (u64::MAX, 1),
            (u64::MAX, u64::MAX),
            (1 << 63, 7),
            (u64::MAX, 97),
            (u32::MAX as u64 * u32::MAX as u64, 3),
        ] {
            for op in 0..5 {
                check(&mut c, a, b, op);
            }
        }
        let mut state = 0x915a_6a73_d4c8_2391u64;
        for _ in 0..12 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = state;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = state;
            for op in 0..8 {
                check(&mut c, a, b, op);
            }
        }
        let l = c.netlist.clone().unwrap();
        drive(&mut c, 9, 3, 0);
        let out = l.result[0];
        c.wires.retain(|w| w.to != out);
        c.connect(l.a[0], out, 0).unwrap();
        assert_eq!(
            word(&c.evaluate().unwrap(), &l.result),
            13,
            "editing a real output wire must change the answer"
        );
        c.remove(l.a[0]);
        assert!(c.evaluate().unwrap().values[&out].is_none());
    }
}

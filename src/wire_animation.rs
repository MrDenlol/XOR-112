use super::*;

/// Distance along the curve, rather than the Bezier parameter, keeps light speed
/// constant. Clip before generating pulses so long ALU connections stay cheap.
pub(super) fn draw_signal_wire(
    painter: &egui::Painter,
    a: Pos2,
    b: Pos2,
    stroke: Stroke,
    animated: bool,
    time: f64,
    seed: f64,
) {
    draw_wire(painter, a, b, stroke);
    if !animated {
        return;
    }
    let points = wire_points(a, b);
    let clip = painter.clip_rect().expand(3.0);
    let phase = ((time * 72.0 + seed * 13.0).rem_euclid(210.0)) as f32;
    let mut distance = 0.0;
    let mut visible = false;
    for pair in points.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        let delta = b - a;
        let length = delta.length();
        if length < 0.001 {
            continue;
        }
        if let Some((lo, hi)) = clip_segment(a, b, clip) {
            visible = true;
            let start = distance + lo * length;
            let end = distance + hi * length;
            let mut head = ((start - phase) / 210.0).floor() * 210.0 + phase;
            while head - 12.0 <= end {
                for band in 0..6 {
                    let left = (head - 12.0 + band as f32 * 4.0).max(start);
                    let right = (head - 8.0 + band as f32 * 4.0).min(end);
                    if left < right {
                        let alpha = (((band as f32 + 0.5) / 6.0 * std::f32::consts::PI).sin()
                            * 180.0) as u8;
                        painter.line_segment(
                            [
                                a + delta * ((left - distance) / length),
                                a + delta * ((right - distance) / length),
                            ],
                            Stroke::new(
                                (stroke.width * 0.58).clamp(0.65, 2.0),
                                Color32::from_white_alpha(alpha),
                            ),
                        );
                    }
                }
                head += 210.0;
            }
        }
        distance += length;
    }
    if visible {
        painter
            .ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
}

fn clip_segment(a: Pos2, b: Pos2, r: Rect) -> Option<(f32, f32)> {
    let d = b - a;
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    for (p, delta, min, max) in [
        (a.x, d.x, r.left(), r.right()),
        (a.y, d.y, r.top(), r.bottom()),
    ] {
        if delta.abs() < 0.00001 {
            if p < min || p > max {
                return None;
            }
        } else {
            let u = (min - p) / delta;
            let v = (max - p) / delta;
            lo = lo.max(u.min(v));
            hi = hi.min(u.max(v));
        }
    }
    (lo <= hi).then_some((lo, hi))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn render(time: f64, enabled: bool, far: bool) -> Vec<egui::epaint::ClippedShape> {
        let ctx = egui::Context::default();
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(500., 200.))),
                ..Default::default()
            },
            |ctx| {
                let painter = ctx.layer_painter(egui::LayerId::background());
                draw_signal_wire(
                    &painter,
                    pos2(if far { -100000. } else { 20. }, 100.),
                    pos2(if far { 100000. } else { 480. }, 100.),
                    Stroke::new(2.6_f32, high()),
                    enabled,
                    time,
                    0.,
                );
            },
        )
        .shapes
    }
    #[test]
    fn light_moves_forward_and_disabled_wires_stay_static() {
        let a = render(0., true, false);
        let b = render(0.2, true, false);
        let white_center = |s: &[egui::epaint::ClippedShape]| {
            s.iter()
                .filter_map(|s| {
                    if let egui::Shape::LineSegment { points, .. } = s.shape {
                        Some((points[0].x + points[1].x) * 0.5)
                    } else {
                        None
                    }
                })
                .find(|x| *x > 150. && *x < 300.)
                .unwrap()
        };
        assert!(white_center(&b) > white_center(&a));
        assert!(a.len() > render(0., false, false).len());
        assert_eq!(
            format!("{:?}", render(0., false, false)),
            format!("{:?}", render(1., false, false))
        );
        assert!(
            render(0., true, true).len() < 100,
            "clip long wires before drawing light"
        );
    }
}

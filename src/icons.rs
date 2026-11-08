use eframe::egui::{self, pos2, vec2, Color32, Rect, Response, Sense, Stroke};

#[derive(Clone, Copy)]
pub enum Icon {
    New,
    Open,
    Save,
    More,
    Undo,
    Redo,
    Delete,
    Copy,
    Table,
    Fit,
    Plus,
    Minus,
}

/// Vector icons stay sharp at different Windows display scales.
pub fn button(ui: &mut egui::Ui, icon: Icon, label: &str, enabled: bool) -> Response {
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            let (rect, response) = ui.allocate_exact_size(vec2(36.0, 38.0), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
            });
            let visuals = ui.style().interact(&response);
            if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
                ui.painter().rect(
                    rect.shrink(1.0),
                    9.0,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            let color = if ui.is_enabled() {
                ui.style().visuals.selection.stroke.color
            } else {
                ui.style().visuals.weak_text_color()
            };
            paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), vec2(23.0, 23.0)),
                icon,
                color,
            );
            response
        })
        .inner;
    response.on_hover_text(label)
}

fn paint(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * rect.width() / 24.0;
    let stroke = Stroke::new(1.7_f32, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([at(a.0, a.1), at(b.0, b.1)], stroke);
    };
    let path = |points: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            points.iter().map(|&(x, y)| at(x, y)).collect(),
            stroke,
        ));
    };
    let box_at = |a: (f32, f32), b: (f32, f32), radius: f32| {
        p.rect_stroke(
            Rect::from_min_max(at(a.0, a.1), at(b.0, b.1)),
            radius,
            stroke,
            egui::StrokeKind::Inside,
        );
    };
    match icon {
        Icon::New => {
            path(&[
                (14., 2.),
                (4., 2.),
                (4., 22.),
                (20., 22.),
                (20., 8.),
                (14., 2.),
                (14., 8.),
                (20., 8.),
            ]);
            line((8., 15.), (16., 15.));
            line((12., 11.), (12., 19.));
        }
        Icon::Open => {
            path(&[
                (2., 19.),
                (2., 5.),
                (9., 5.),
                (12., 8.),
                (20., 8.),
                (20., 11.),
            ]);
            path(&[(2., 20.), (6., 11.), (23., 11.), (19., 20.), (2., 20.)]);
        }
        Icon::Save => {
            path(&[
                (3., 3.),
                (18., 3.),
                (21., 6.),
                (21., 21.),
                (3., 21.),
                (3., 3.),
            ]);
            box_at((7., 3.), (16., 10.), 1.);
            box_at((7., 14.), (17., 21.), 1.);
            line((13., 4.), (13., 8.));
        }
        Icon::More => {
            for x in [5., 12., 19.] {
                p.circle_filled(at(x, 12.), 1.7, color);
            }
        }
        Icon::Undo | Icon::Redo => {
            let flip = |x: f32| {
                if matches!(icon, Icon::Redo) {
                    24. - x
                } else {
                    x
                }
            };
            let points = [(4., 9.), (13., 9.), (18., 12.), (19., 16.), (16., 20.)];
            path(&points.map(|(x, y)| (flip(x), y)));
            path(&[(flip(9.), 4.), (flip(4.), 9.), (flip(9.), 14.)]);
        }
        Icon::Delete => {
            line((3., 6.), (21., 6.));
            path(&[(6., 6.), (7., 21.), (17., 21.), (18., 6.)]);
            path(&[(9., 6.), (9., 3.), (15., 3.), (15., 6.)]);
            line((10., 10.), (10., 17.));
            line((14., 10.), (14., 17.));
        }
        Icon::Copy => {
            box_at((8., 8.), (21., 22.), 2.);
            path(&[(16., 5.), (16., 2.), (3., 2.), (3., 16.), (5., 16.)]);
        }
        Icon::Table => {
            box_at((2., 3.), (22., 21.), 2.);
            line((2., 9.), (22., 9.));
            line((2., 15.), (22., 15.));
            line((9., 3.), (9., 21.));
            line((16., 3.), (16., 21.));
        }
        Icon::Fit => {
            for (x, y, dx, dy) in [
                (3., 3., 1., 1.),
                (21., 3., -1., 1.),
                (3., 21., 1., -1.),
                (21., 21., -1., -1.),
            ] {
                path(&[(x, y + dy * 5.), (x, y), (x + dx * 5., y)]);
            }
            box_at((8., 8.), (16., 16.), 2.);
        }
        Icon::Plus => {
            line((4., 12.), (20., 12.));
            line((12., 4.), (12., 20.));
        }
        Icon::Minus => line((4., 12.), (20., 12.)),
    }
}

// Shared vector paths for the XOR symbol in the ribbon and native window icon.
fn xor_paths() -> Vec<Vec<egui::Pos2>> {
    let mut paths = Vec::new();
    for points in [
        [pos2(10., 8.), pos2(19., 8.), pos2(23., 11.), pos2(26., 16.)],
        [
            pos2(26., 16.),
            pos2(23., 21.),
            pos2(19., 24.),
            pos2(10., 24.),
        ],
        [
            pos2(10., 24.),
            pos2(15., 19.),
            pos2(15., 13.),
            pos2(10., 8.),
        ],
        [pos2(6., 24.), pos2(11., 19.), pos2(11., 13.), pos2(6., 8.)],
    ] {
        paths.push(
            (0..=32)
                .map(|i| {
                    let t = i as f32 / 32.;
                    let u = 1. - t;
                    (points[0].to_vec2() * u * u * u
                        + points[1].to_vec2() * 3. * u * u * t
                        + points[2].to_vec2() * 3. * u * t * t
                        + points[3].to_vec2() * t * t * t)
                        .to_pos2()
                })
                .collect(),
        );
    }
    paths.push(vec![pos2(26., 16.), pos2(29., 16.)]);
    paths
}
pub fn logo(p: &egui::Painter, rect: Rect) {
    p.rect_filled(rect, 9.0, Color32::from_rgb(33, 115, 70));
    for path in xor_paths() {
        p.add(egui::Shape::line(
            path.into_iter()
                .map(|v| rect.min + v.to_vec2() * rect.width() / 32.0)
                .collect(),
            Stroke::new(1.6 * rect.width() / 32.0, Color32::WHITE),
        ));
    }
}
pub fn app_icon() -> egui::IconData {
    let paths = xor_paths();
    let mut rgba = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64 {
        for x in 0..64 {
            let mut sum = [0u32; 4];
            for sy in 0..4 {
                for sx in 0..4 {
                    let p = pos2(
                        (x as f32 + (sx as f32 + 0.5) / 4.) / 2.,
                        (y as f32 + (sy as f32 + 0.5) / 4.) / 2.,
                    );
                    let inside = Rect::from_min_max(pos2(1., 1.), pos2(31., 31.)).contains(p)
                        && (p - pos2(p.x.clamp(7., 25.), p.y.clamp(7., 25.))).length() <= 6.;
                    let ink = paths.iter().any(|path| {
                        path.windows(2).any(|s| {
                            let v = s[1] - s[0];
                            let t = ((p - s[0]).dot(v) / v.length_sq().max(0.0001)).clamp(0., 1.);
                            p.distance(s[0] + v * t) < 0.8
                        })
                    });
                    let color = if !inside {
                        [0, 0, 0, 0]
                    } else if ink {
                        [255, 255, 255, 255]
                    } else {
                        [33, 115, 70, 255]
                    };
                    for (a, b) in sum.iter_mut().zip(color) {
                        *a += b;
                    }
                }
            }
            rgba.extend(sum.map(|v| (v / 16) as u8));
        }
    }
    egui::IconData {
        rgba,
        width: 64,
        height: 64,
    }
}

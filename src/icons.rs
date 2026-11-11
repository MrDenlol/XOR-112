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
    Home,
    Settings,
    Help,
    Library,
    Info,
    Pulse,
    Check,
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

pub fn paint(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
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
        Icon::Home => {
            // Familiar house silhouette, with a single doorway.
            path(&[(3., 10.), (12., 3.), (21., 10.)]);
            path(&[
                (5., 9.),
                (5., 21.),
                (10., 21.),
                (10., 14.),
                (14., 14.),
                (14., 21.),
                (19., 21.),
                (19., 9.),
            ]);
        }
        Icon::Settings => {
            // One closed cog outline with eight equally spaced teeth.
            let mut points = Vec::new();
            for tooth in 0..8 {
                for (offset, radius) in [
                    (-0.5, 7.5),
                    (-0.25, 7.5),
                    (-0.25, 10.),
                    (0.25, 10.),
                    (0.25, 7.5),
                    (0.5, 7.5),
                ] {
                    let angle = (tooth as f32 + offset) * std::f32::consts::TAU / 8.;
                    points.push(at(12. + radius * angle.cos(), 12. + radius * angle.sin()));
                }
            }
            p.add(egui::Shape::closed_line(points, stroke));
            p.circle_stroke(at(12., 12.), rect.width() * 3.2 / 24., stroke);
        }
        Icon::Help => {
            p.circle_stroke(at(12., 12.), rect.width() * 0.4, stroke);
            for points in [
                [(8.5, 8.4), (8.5, 4.7), (15.8, 4.7), (15.8, 8.4)],
                [(15.8, 8.4), (15.8, 11.5), (12., 11.1), (12., 14.)],
            ] {
                p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                    points.map(|(x, y)| at(x, y)),
                    false,
                    Color32::TRANSPARENT,
                    stroke,
                ));
            }
            for (x, y) in [(8.5, 8.4), (12., 14.)] {
                p.circle_filled(at(x, y), stroke.width / 2., color);
            }
            p.circle_filled(at(12., 18.), rect.width() / 24., color);
        }
        Icon::Info => {
            let fill = Color32::from_rgb(
                (255. * 0.9 + color.r() as f32 * 0.1) as u8,
                (255. * 0.9 + color.g() as f32 * 0.1) as u8,
                (255. * 0.9 + color.b() as f32 * 0.1) as u8,
            );
            p.circle_filled(at(12., 12.), rect.width() * 0.43, fill);
            p.circle_stroke(at(12., 12.), rect.width() * 0.43, stroke);
            p.circle_filled(at(12., 7.), rect.width() * 1.1 / 24., color);
            path(&[(10., 10.), (12., 10.), (12., 17.)]);
            line((9., 17.), (15., 17.));
        }
        Icon::Library => {
            // Identical square geometry aligned to physical pixels, even at 125% DPI.
            let pixels = p.ctx().pixels_per_point();
            let edge = (rect.width() * 7. / 24. * pixels).round() / pixels;
            let gap = (rect.width() * 4. / 24. * pixels).round() / pixels;
            let origin = rect.center() - vec2(edge + gap * 0.5, edge + gap * 0.5);
            let origin = pos2(
                (origin.x * pixels).round() / pixels,
                (origin.y * pixels).round() / pixels,
            );
            for x in 0..2 {
                for y in 0..2 {
                    p.rect_stroke(
                        Rect::from_min_size(
                            origin + vec2(x as f32, y as f32) * (edge + gap),
                            vec2(edge, edge),
                        ),
                        0,
                        Stroke::new((1.5 * pixels).round() / pixels, color),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
        Icon::Pulse => path(&[
            (2., 13.),
            (7., 13.),
            (10., 5.),
            (13., 20.),
            (16., 10.),
            (18., 13.),
            (22., 13.),
        ]),
        Icon::Check => path(&[(5., 12.), (10., 17.), (20., 6.)]),
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
        Icon::More => path(&[(7., 9.), (12., 14.), (17., 9.)]),
        Icon::Undo | Icon::Redo => {
            let flip = |x: f32| {
                if matches!(icon, Icon::Redo) {
                    24. - x
                } else {
                    x
                }
            };
            let curve = [
                at(flip(5.), 9.),
                at(flip(20.), 5.),
                at(flip(23.), 20.),
                at(flip(12.), 20.),
            ];
            p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                curve,
                false,
                Color32::TRANSPARENT,
                stroke,
            ));
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
            // Equal cells; internal strokes stop at the centered outer border.
            let pixels = p.ctx().pixels_per_point();
            let cell = (rect.width() * 6. / 24. * pixels).round() / pixels;
            let center = rect.center();
            let origin = pos2(
                ((center.x - cell * 1.5) * pixels).round() / pixels,
                ((center.y - cell * 1.5) * pixels).round() / pixels,
            );
            let bounds = Rect::from_min_size(origin, vec2(cell * 3., cell * 3.));
            let grid_stroke = Stroke::new((1.5 * pixels).round() / pixels, color);
            p.rect_stroke(bounds, 0, grid_stroke, egui::StrokeKind::Middle);
            for i in 1..3 {
                let offset = i as f32 * cell;
                p.line_segment(
                    [origin + vec2(offset, 0.), origin + vec2(offset, cell * 3.)],
                    grid_stroke,
                );
                p.line_segment(
                    [origin + vec2(0., offset), origin + vec2(cell * 3., offset)],
                    grid_stroke,
                );
            }
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
    }
}

// Shared vector paths for the XOR symbol in the ribbon and native window icon.
pub fn command(
    ui: &mut egui::Ui,
    icon: Icon,
    caption: &str,
    hint: &str,
    enabled: bool,
    width: f32,
) -> Response {
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            let (rect, response) = ui.allocate_exact_size(vec2(width, 58.), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, caption)
            });
            let visuals = ui.style().interact(&response);
            if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
                ui.painter().rect(
                    rect.shrink(1.),
                    8.,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            let color = if enabled {
                ui.visuals().selection.stroke.color
            } else {
                ui.visuals().weak_text_color()
            };
            paint(
                ui.painter(),
                Rect::from_center_size(rect.center_top() + vec2(0., 19.), vec2(24., 24.)),
                icon,
                color,
            );
            ui.painter().text(
                rect.center_top() + vec2(0., 45.),
                egui::Align2::CENTER_CENTER,
                caption,
                egui::FontId::proportional(11.),
                if enabled {
                    ui.visuals().text_color()
                } else {
                    color
                },
            );
            response
        })
        .inner;
    response.on_hover_text(hint)
}

pub fn tab(ui: &mut egui::Ui, icon: Icon, caption: &str, active: bool, width: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 36.), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, caption)
    });
    let accent = ui.visuals().selection.stroke.color;
    if active || response.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(vec2(2., 2.)),
            8.,
            if active {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.hovered.bg_fill
            },
        );
    }
    paint(
        ui.painter(),
        Rect::from_center_size(rect.left_center() + vec2(23., 0.), vec2(20., 20.)),
        icon,
        if active {
            accent
        } else {
            ui.visuals().text_color()
        },
    );
    ui.painter().text(
        rect.left_center() + vec2(42., 0.),
        egui::Align2::LEFT_CENTER,
        caption,
        egui::FontId::proportional(14.),
        ui.visuals().text_color(),
    );
    if active {
        ui.painter().line_segment(
            [
                rect.left_bottom() + vec2(2., 0.),
                rect.right_bottom() - vec2(2., 0.),
            ],
            Stroke::new(2.5_f32, accent),
        );
    }
    response
}

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
pub fn logo(p: &egui::Painter, rect: Rect, color: Color32) {
    p.rect_filled(rect, 9.0, color);
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
    app_icon_colored(Color32::from_rgb(33, 115, 70))
}
pub fn app_icon_colored(background: Color32) -> egui::IconData {
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
                        [background.r(), background.g(), background.b(), 255]
                    };
                    for (a, b) in sum.iter_mut().zip(color) {
                        *a += b as u32;
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

use super::*;
use std::cell::Cell;
thread_local! {static ACTIVE:Cell<Theme>=const{Cell::new(Theme::Green)};}
pub(super) fn set_theme(t: Theme) {
    ACTIVE.with(|v| v.set(t));
}
pub(super) fn accent() -> Color32 {
    ACTIVE.with(|t| match t.get() {
        Theme::Green => Color32::from_rgb(33, 115, 70),
        Theme::Blue => Color32::from_rgb(36, 101, 180),
        Theme::Gray => Color32::from_rgb(91, 100, 109),
    })
}
pub(super) fn tint(c: Color32) -> Color32 {
    ACTIVE.with(|t| match t.get() {
        Theme::Green => c,
        Theme::Blue => Color32::from_rgba_unmultiplied(
            c.r().saturating_sub(3),
            c.b().saturating_add(4),
            c.g().saturating_add(8),
            c.a(),
        ),
        Theme::Gray => {
            let v = ((c.r() as u16 + c.g() as u16 + c.b() as u16) / 3) as u8;
            Color32::from_rgba_unmultiplied(v, v, v, c.a())
        }
    })
}
pub(super) fn ink() -> Color32 {
    tint(Color32::from_rgb(39, 57, 48))
}
pub(super) fn muted() -> Color32 {
    tint(Color32::from_rgb(116, 132, 122))
}
pub(super) fn paper() -> Color32 {
    tint(Color32::from_rgb(248, 250, 248))
}
pub(super) fn panel() -> Color32 {
    Color32::from_rgb(254, 255, 254)
}
pub(super) fn border() -> Color32 {
    tint(Color32::from_rgb(220, 230, 223))
}
pub(super) fn high() -> Color32 {
    Color32::from_rgb(42, 142, 113)
}
pub(super) fn low() -> Color32 {
    Color32::from_rgb(142, 159, 148)
}
pub(super) fn error() -> Color32 {
    Color32::from_rgb(183, 106, 58)
}
pub(super) fn apply(ctx: &egui::Context, t: Theme) {
    set_theme(t);
    let mut style = (*ctx.style()).clone();
    style.visuals.override_text_color = Some(ink());
    style.visuals.panel_fill = panel();
    style.visuals.window_fill = panel();
    style.visuals.faint_bg_color = paper();
    style.visuals.selection.bg_fill = tint(Color32::from_rgb(216, 237, 223));
    style.visuals.selection.stroke = Stroke::new(1.0_f32, accent());
    style.visuals.widgets.inactive.bg_fill = tint(Color32::from_rgb(239, 245, 241));
    style.visuals.widgets.inactive.weak_bg_fill = tint(Color32::from_rgb(239, 245, 241));
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border());
    style.visuals.widgets.hovered.bg_fill = tint(Color32::from_rgb(221, 239, 227));
    style.visuals.widgets.hovered.weak_bg_fill = style.visuals.widgets.hovered.bg_fill;
    style.visuals.widgets.active.bg_fill = tint(Color32::from_rgb(211, 231, 219));
    style.visuals.widgets.active.weak_bg_fill = style.visuals.widgets.active.bg_fill;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, accent());
    ctx.set_style_of(egui::Theme::Light, style);
}

use eframe::egui::{self, Color32, CornerRadius, Margin, Shadow, Stroke, Style, Visuals};

// -- Brand palette ----------------------------------------------------------

/// Deep blue-black background
pub const BG_DARK: Color32 = Color32::from_rgb(18, 18, 24);
/// Slightly lighter panel background
pub const BG_PANEL: Color32 = Color32::from_rgb(24, 24, 32);
/// Faint row stripe
pub const BG_STRIPE: Color32 = Color32::from_rgb(30, 30, 40);
/// Extreme (text-edit / scrollbar track)
pub const BG_EXTREME: Color32 = Color32::from_rgb(12, 12, 16);

/// Primary accent (teal / cyan)
pub const ACCENT: Color32 = Color32::from_rgb(0, 188, 212);
/// Lighter accent for hover
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(38, 210, 230);
/// Accent for selection highlight
pub const ACCENT_SELECT: Color32 = Color32::from_rgb(0, 188, 212);

/// Green for "good" values
pub const GREEN: Color32 = Color32::from_rgb(76, 175, 80);
/// Yellow for "warn" values
pub const YELLOW: Color32 = Color32::from_rgb(255, 193, 7);
/// Red for "error" values
pub const RED: Color32 = Color32::from_rgb(244, 67, 54);
/// Orange for warning text
pub const ORANGE: Color32 = Color32::from_rgb(255, 152, 0);

/// Primary text (near-white)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(224, 224, 230);
/// Secondary / weak text
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(140, 140, 160);
/// Monospace code background
pub const CODE_BG: Color32 = Color32::from_rgb(35, 35, 48);

// -- Widget visuals ---------------------------------------------------------

fn widget_visuals(
    bg: Color32,
    bg_stroke: Color32,
    fg: Color32,
    rounding: u8,
    expansion: f32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: Stroke::new(1.0, bg_stroke),
        corner_radius: CornerRadius::same(rounding),
        fg_stroke: Stroke::new(1.0, fg),
        expansion,
    }
}

// -- Build full style -------------------------------------------------------

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    style.spacing.indent = 18.0;
    style.spacing.slider_width = 160.0;
    style.spacing.combo_width = 200.0;
    style.spacing.scroll.bar_width = 8.0;

    // Visuals
    let mut visuals = Visuals::dark();

    visuals.dark_mode = true;
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_DARK;
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(50, 50, 65));
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(80),
    };

    visuals.faint_bg_color = BG_STRIPE;
    visuals.extreme_bg_color = BG_EXTREME;
    visuals.code_bg_color = CODE_BG;

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = ORANGE;
    visuals.error_fg_color = RED;

    visuals.selection.bg_fill = ACCENT_SELECT.gamma_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.indent_has_left_vline = true;
    visuals.collapsing_header_frame = true;

    // Widget states
    visuals.widgets.noninteractive = widget_visuals(
        BG_PANEL,
        Color32::from_rgb(50, 50, 65),
        TEXT_SECONDARY,
        4,
        0.0,
    );
    visuals.widgets.inactive = widget_visuals(
        Color32::from_rgb(40, 40, 55),
        Color32::from_rgb(60, 60, 80),
        TEXT_PRIMARY,
        6,
        0.0,
    );
    visuals.widgets.hovered = widget_visuals(
        Color32::from_rgb(50, 50, 70),
        ACCENT_HOVER,
        TEXT_PRIMARY,
        6,
        1.0,
    );
    visuals.widgets.active = widget_visuals(
        Color32::from_rgb(35, 35, 50),
        ACCENT,
        Color32::WHITE,
        6,
        0.0,
    );
    visuals.widgets.open =
        widget_visuals(Color32::from_rgb(45, 45, 60), ACCENT, TEXT_PRIMARY, 6, 0.0);

    visuals.popup_shadow = Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: Color32::from_black_alpha(60),
    };

    visuals.menu_corner_radius = CornerRadius::same(6);

    style.visuals = visuals;

    // Override text styles for slightly larger defaults
    use egui::FontId;
    use egui::TextStyle;
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(18.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(12.5));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.0));

    ctx.set_style(style);
}

// -- Utility helpers --------------------------------------------------------

/// Color a latency value: green < 10 ms, yellow < 30 ms, red >= 30 ms
pub fn latency_color(ms: f32) -> Color32 {
    if ms < 10.0 {
        GREEN
    } else if ms < 30.0 {
        YELLOW
    } else {
        RED
    }
}

/// Render a small colored badge (colored background + white text)
pub fn badge(ui: &mut egui::Ui, label: &str, bg: Color32) {
    let padding = Margin::symmetric(6, 2);
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(3))
        .inner_margin(padding)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label)
                    .small()
                    .strong()
                    .color(Color32::WHITE),
            );
        });
}

/// A section header inside the side panel
pub fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(text).size(13.0).strong().color(ACCENT));
    ui.add_space(2.0);
}

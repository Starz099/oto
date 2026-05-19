use eframe::egui;

pub struct Theme {
    pub bg_dark: egui::Color32,
    pub bg_panel: egui::Color32,
    pub bg_selection: egui::Color32,
    pub primary_pink: egui::Color32,
    pub text_main: egui::Color32,
    pub text_dim: egui::Color32,
    pub text_accent: egui::Color32,
    
    pub spacing_outer: f32,
    pub item_spacing: f32,
    pub corner_radius: f32,
    
    pub font_size_heading: f32,
    pub font_size_body: f32,
    pub font_size_small: f32,
}

impl Theme {
    pub fn pastel_pink() -> Self {
        Self {
            bg_dark: egui::Color32::from_rgba_premultiplied(15, 15, 18, 235), // Semi-transparent
            bg_panel: egui::Color32::from_rgba_premultiplied(22, 22, 26, 245),
            bg_selection: egui::Color32::from_rgba_premultiplied(35, 30, 35, 255),
            primary_pink: egui::Color32::from_rgb(248, 200, 220), // Pastel Pink
            text_main: egui::Color32::from_rgb(235, 235, 240),
            text_dim: egui::Color32::from_rgb(150, 150, 160),
            text_accent: egui::Color32::from_rgb(255, 182, 193), // Light Pink
            
            spacing_outer: 24.0,
            item_spacing: 12.0,
            corner_radius: 10.0,
            
            font_size_heading: 26.0,
            font_size_body: 17.0,
            font_size_small: 14.0,
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let mut style = (*ctx.global_style()).clone();
    
    // Custom Font Sizes
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(theme.font_size_heading, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(theme.font_size_body, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(theme.font_size_body, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Small, egui::FontId::new(theme.font_size_small, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(theme.font_size_small, egui::FontFamily::Monospace));
    
    style.spacing.item_spacing = egui::vec2(theme.item_spacing, theme.item_spacing);
    style.spacing.window_margin = egui::Margin::same(theme.spacing_outer as i8);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    
    ctx.set_global_style(style);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::TRANSPARENT;
    visuals.window_fill = egui::Color32::TRANSPARENT;
    
    // Selection & Accents
    visuals.selection.bg_fill = theme.primary_pink.gamma_multiply(0.2);
    visuals.selection.stroke = egui::Stroke::new(1.5, theme.primary_pink);
    
    // Widgets
    visuals.widgets.inactive.bg_fill = theme.bg_panel;
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(theme.corner_radius as u8);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.text_dim);
    
    visuals.widgets.hovered.bg_fill = theme.bg_selection;
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(theme.corner_radius as u8);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, theme.primary_pink);
    
    visuals.widgets.active.bg_fill = theme.primary_pink;
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(theme.corner_radius as u8);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, theme.primary_pink);
    
    // Separators derive from noninteractive stroke
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, theme.bg_selection);

    // Sliders
    visuals.widgets.inactive.expansion = 0.0;
    visuals.widgets.active.expansion = 1.0;

    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 12],
        blur: 32,
        spread: 0,
        color: egui::Color32::from_black_alpha(200),
    };

    ctx.set_visuals(visuals);
}

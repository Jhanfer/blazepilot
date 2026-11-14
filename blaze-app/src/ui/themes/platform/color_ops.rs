//! Operaciones de color centralizadas.

use egui::{Color32, Rgba, ecolor::Hsva};

/// Indica si un color de fondo es oscuro
///
/// Se usa para decidir si un hover debe aclarar u oscurecer
pub fn is_dark(color: Color32) -> bool {
    relative_luminance(color) < 0.5
}

/// Luminancia relativa de un color
pub fn relative_luminance(color: Color32) -> f32 {
    let [r, g, b, _] = color.to_srgba_unmultiplied();

    fn channel(value: u8) -> f32 {
        let value = value as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// Relación de contraste WCAG entre dos colores (1.0 = idénticos, 21.0 = máximo)
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

/// Mezcla `a` y `b` en espacio lineal. `t = 0` devuelve `a`, `t = 1` devuelve `b`
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let ra = Rgba::from(a);
    let rb = Rgba::from(b);

    let channel = |index: usize| ra[index] * (1.0 - t) + rb[index] * t;

    Color32::from(Rgba::from_rgba_premultiplied(
        channel(0),
        channel(1),
        channel(2),
        channel(3),
    ))
}

/// Reemplaza el alpha de un color conservando su RGB
pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

/// Multiplica el alpha de un color (0.0 = transparente, 1.0 = sin cambios)
pub fn scale_alpha(color: Color32, factor: f32) -> Color32 {
    let alpha = (color.a() as f32 * factor.clamp(0.0, 1.0)).round() as u8;
    with_alpha(color, alpha)
}

/// Aclara u oscurece un color desplazando su luminosidad HSV
pub fn shift_lightness(color: Color32, delta: f32) -> Color32 {
    let mut hsva = Hsva::from(color);
    hsva.v = (hsva.v + delta).clamp(0.0, 1.0);
    hsva.into()
}

/// Color de superficie elevada a partir del fondo del panel
pub fn elevate(background: Color32, background_is_dark: bool) -> Color32 {
    if background_is_dark {
        shift_lightness(background, 0.06)
    } else {
        shift_lightness(background, -0.04)
    }
}

/// Color de hover a partir del fondo base y el color de contenido
pub fn hover_from(background: Color32, content: Color32) -> Color32 {
    mix(background, content, 0.08)
}

/// Color de selección a partir del fondo base y el acento
pub fn selected_from(background: Color32, accent: Color32) -> Color32 {
    mix(background, accent, 0.22)
}

/// Oscurece un color en espacio gamma
pub fn shade(color: Color32) -> Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(
        (r as f32 * 0.70) as u8,
        (g as f32 * 0.70) as u8,
        (b as f32 * 0.70) as u8,
        a,
    )
}

/// Texto legible (blanco o casi negro) sobre un color de fondo
pub fn contrast_color(background: Color32) -> Color32 {
    let [r, g, b, _] = background.to_srgba_unmultiplied();
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

    if luminance > 0.5 {
        Color32::from_rgb(30, 30, 30)
    } else {
        Color32::WHITE
    }
}

/// Garantiza que `foreground` tenga al menos `min_ratio` de contraste sobre el fondo
///
/// Si el color original ya cumple, se devuelve tal cual. Si no, se prueban
/// variantes hacia blanco y hacia negro y se elige la mejor
pub fn ensure_contrast(foreground: Color32, background: Color32, min_ratio: f32) -> Color32 {
    let current = contrast_ratio(foreground, background);
    if current >= min_ratio {
        return foreground;
    }

    let mut best = foreground;
    let mut best_ratio = current;

    for step in 1..=20 {
        let t = step as f32 / 20.0;

        for candidate in [
            mix(foreground, Color32::BLACK, t),
            mix(foreground, Color32::WHITE, t),
        ] {
            let ratio = contrast_ratio(candidate, background);
            if ratio > best_ratio {
                best = candidate;
                best_ratio = ratio;
            }
        }

        if best_ratio >= min_ratio {
            break;
        }
    }

    if best_ratio >= min_ratio {
        best
    } else {
        contrast_color(background)
    }
}

/// Ajusta un color para que no quede por debajo de una luminosidad HSL mínima
pub fn ensure_min_lightness_hsl(color: Color32, min_lightness: f32) -> Color32 {
    let [r8, g8, b8, a8] = color.to_srgba_unmultiplied();

    let r = r8 as f32 / 255.0;
    let g = g8 as f32 / 255.0;
    let b = b8 as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if l >= min_lightness {
        return color;
    }

    let delta = max - min;
    let s = if delta < 1e-6 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta < 1e-6 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let new_l = min_lightness.clamp(0.0, 1.0);
    let c = (1.0_f32 - (2.0_f32 * new_l - 1.0_f32).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = new_l - c / 2.0;

    let (r1, g1, b1) = match h {
        0.0..=59.0 => (c, x, 0.0),
        60.0..=119.0 => (x, c, 0.0),
        120.0..=179.0 => (0.0, c, x),
        180.0..=239.0 => (0.0, x, c),
        240.0..=299.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color32::from_rgba_unmultiplied(
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
        a8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_ratio_is_symmetric() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(255, 255, 255);
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < f32::EPSILON);
        assert!((contrast_ratio(a, b) - 21.0).abs() < 0.01);
    }

    #[test]
    fn ensure_contrast_returns_readable_text() {
        let bg = Color32::from_rgb(37, 23, 49);
        let fg = Color32::from_rgb(80, 60, 90);
        let fixed = ensure_contrast(fg, bg, 4.5);
        assert!(contrast_ratio(fixed, bg) >= 4.5);
    }

    #[test]
    fn mix_endpoints_are_exact() {
        let a = Color32::from_rgb(10, 20, 30);
        let b = Color32::from_rgb(200, 210, 220);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
    }
}

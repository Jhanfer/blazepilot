use egui::{
    Color32, CornerRadius, Mesh, Painter, Pos2, Rect, Shape, Stroke, epaint::PathShape, pos2,
};

pub const TAB_H: f32 = 40.0;
pub const TAB_MAX_WIDTH: f32 = 150.0;
pub const TAB_MIN_WIDTH: f32 = 100.0;
pub const TAB_SPACING: f32 = 5.0;
pub const MIN_VISIBLE_W: f32 = 1.0;
pub const INACTIVE_MIN_VISIBLE_W: f32 = 40.0;
pub const FLOAT_GAP: f32 = 8.0;
pub const INACTIVE_TOP_INSET: f32 = 1.0;

pub const R_BAR: f32 = 17.0;
pub const R_TAB: f32 = 17.0;
pub const R_INACTIVE: f32 = 17.0;
pub const R_FILLET: f32 = 16.0;

fn arc(c: Pos2, r: f32, from_deg: f32, to_deg: f32, seg: usize) -> Vec<Pos2> {
    (0..=seg)
        .map(|i| {
            let t = i as f32 / seg as f32;
            let a = (from_deg + (to_deg - from_deg) * t).to_radians();
            pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

pub fn draw_tab_merged(painter: &Painter, bar: Rect, tab: Rect, fill: Color32, stroke: Stroke) {
    let rb = R_BAR;
    let rt = R_TAB.min(tab.width() / 2.0);
    let rf = R_FILLET;
    let seg = 10_usize;

    let left_flush = tab.left() - bar.left() < rb + rf;
    let right_flush = bar.right() - tab.right() < rb + rf;

    let mut tab = tab;
    if left_flush {
        tab.min.x = bar.left();
    }
    if right_flush {
        tab.max.x = bar.right();
    }

    if tab.width() > bar.width() {
        tab.max.x = tab.min.x + bar.width();
    }

    let cl = pos2(tab.left() - rf, bar.top() - rf);
    let cr = pos2(tab.right() + rf, bar.top() - rf);

    let bar_radius = CornerRadius {
        nw: if left_flush { 0 } else { rb as u8 },
        ne: if right_flush { 0 } else { rb as u8 },
        sw: rb as u8,
        se: rb as u8,
    };

    painter.rect_filled(bar, bar_radius, fill);

    let tab_fill = Rect::from_min_max(tab.min, pos2(tab.right(), tab.bottom() + 1.0));

    painter.rect_filled(
        tab_fill,
        CornerRadius {
            nw: rt as u8,
            ne: rt as u8,
            sw: 0,
            se: 0,
        },
        fill,
    );

    let mut fillets = vec![];
    if !left_flush {
        fillets.push((cl, pos2(tab.left(), bar.top()), 90.0, 0.0));
    }
    if !right_flush {
        fillets.push((cr, pos2(tab.right(), bar.top()), 180.0, 90.0));
    }

    for (center, corner, from, to) in fillets {
        let pts = arc(center, rf, from, to, seg);

        let mut mesh = Mesh::default();

        mesh.colored_vertex(corner, fill);

        for p in &pts {
            mesh.colored_vertex(*p, fill);
        }
        for i in 1..pts.len() {
            let i = i as u32;
            mesh.add_triangle(0, i, i + 1);
        }

        painter.add(Shape::mesh(mesh));
    }

    let mut p: Vec<Pos2> = Vec::new();
    if !left_flush {
        p.extend(arc(
            pos2(bar.left() + rb, bar.top() + rb),
            rb,
            180.0,
            270.0,
            seg,
        ));
        p.extend(arc(cl, rf, 90.0, 0.0, seg));
    }

    p.extend(arc(
        pos2(tab.left() + rt, tab.top() + rt),
        rt,
        180.0,
        270.0,
        seg,
    ));
    p.extend(arc(
        pos2(tab.right() - rt, tab.top() + rt),
        rt,
        270.0,
        360.0,
        seg,
    ));

    if !right_flush {
        p.extend(arc(cr, rf, 180.0, 90.0, seg));
        p.extend(arc(
            pos2(bar.right() - rb, bar.top() + rb),
            rb,
            270.0,
            360.0,
            seg,
        ));
    }

    p.extend(arc(
        pos2(bar.right() - rb, bar.bottom() - rb),
        rb,
        0.0,
        90.0,
        seg,
    ));
    p.extend(arc(
        pos2(bar.left() + rb, bar.bottom() - rb),
        rb,
        90.0,
        180.0,
        seg,
    ));

    painter.add(PathShape::closed_line(p, stroke));
}

pub fn draw_tab_bar(painter: &Painter, bar: Rect, fill: Color32, stroke: Stroke) {
    let rb = R_BAR;
    let seg = 10;
    let mut p = Vec::new();

    p.extend(arc(
        pos2(bar.right() - rb, bar.top() + rb),
        rb,
        270.0,
        360.0,
        seg,
    ));

    p.extend(arc(
        pos2(bar.right() - rb, bar.bottom() - rb),
        rb,
        0.0,
        90.0,
        seg,
    ));

    p.extend(arc(
        pos2(bar.left() + rb, bar.bottom() - rb),
        rb,
        90.0,
        180.0,
        seg,
    ));

    p.extend(arc(
        pos2(bar.left() + rb, bar.top() + rb),
        rb,
        180.0,
        270.0,
        seg,
    ));

    painter.rect_filled(bar, CornerRadius::same(rb as u8), fill);

    painter.add(PathShape::closed_line(p, stroke));
}

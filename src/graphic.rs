use macroquad::prelude::*;
use smth::{Plot, PlotKind};

pub async fn show_plots(plots: &[Plot]) {
    let mut orbit = OrbitCamera::default();
    loop {
        clear_background(Color::from_rgba(250, 251, 253, 255));
        for plot in plots {
            if plot.dimension == 3 { draw_3d(plot, &mut orbit); } else { draw_2d(plot); }
        }
        if is_key_pressed(KeyCode::Escape) || is_quit_requested() { break; }
        next_frame().await;
    }
}

#[derive(Debug)]
struct OrbitCamera { yaw: f32, pitch: f32, distance: f32 }
impl Default for OrbitCamera {
    fn default() -> Self { Self { yaw: 0.8, pitch: 0.65, distance: 18.0 } }
}

fn draw_2d(plot: &Plot) {
    let left = 64.0;
    let top = 42.0;
    let width = (screen_width() - left - 24.0).max(1.0);
    let height = (screen_height() - top - 48.0).max(1.0);
    let (xmin, xmax) = plot.x_range;
    let (ymin, ymax) = plot.y_range;
    let corners = [
        plot.transform(xmin, ymin, 0.0),
        plot.transform(xmin, ymax, 0.0),
        plot.transform(xmax, ymin, 0.0),
        plot.transform(xmax, ymax, 0.0),
    ];
    let wxmin = corners.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let wxmax = corners
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    let wymin = corners.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let wymax = corners
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let screen = |x: f64, y: f64| {
        let p = plot.transform(x, y, 0.0);
        vec2(
            left + ((p[0] - wxmin) / (wxmax - wxmin).max(1e-12)) as f32 * width,
            top + (1.0 - ((p[1] - wymin) / (wymax - wymin).max(1e-12)) as f32) * height,
        )
    };
    if xmin <= 0.0 && xmax >= 0.0 {
        let a = screen(0.0, ymin);
        let b = screen(0.0, ymax);
        draw_line(a.x, a.y, b.x, b.y, 1.0, GRAY);
    }
    if ymin <= 0.0 && ymax >= 0.0 {
        let a = screen(xmin, 0.0);
        let b = screen(xmax, 0.0);
        draw_line(a.x, a.y, b.x, b.y, 1.0, GRAY);
    }

    let nx = plot.resolution.max(2);
    let dx = (xmax - xmin) / (nx - 1) as f64;
    if plot.kind == PlotKind::Function {
        let mut previous: Option<Vec2> = None;
        for i in 0..nx {
            let x = xmin + dx * i as f64;
            let point = plot
                .evaluate(x, 0.0, 0.0)
                .ok()
                .filter(|y| *y >= ymin && *y <= ymax)
                .map(|y| screen(x, y));
            if let (Some(a), Some(b)) = (previous, point) {
                draw_line(a.x, a.y, b.x, b.y, 2.0, BLUE);
            }
            previous = point;
        }
    } else {
        let ny = nx;
        let dy = (ymax - ymin) / (ny - 1) as f64;
        let tolerance = dx.hypot(dy) * 0.6;
        for ix in 0..nx {
            for iy in 0..ny {
                let x = xmin + dx * ix as f64;
                let y = ymin + dy * iy as f64;
                let included = if plot.kind == PlotKind::ImplicitCurve
                    && matches!(
                        plot.expression,
                        smth::Expr::Equals(_, _) | smth::Expr::NotEquals(_, _)
                    ) {
                    plot.residual(x, y, 0.0).is_ok_and(|v| v.abs() <= tolerance)
                } else {
                    plot.satisfies(x, y, 0.0, tolerance).unwrap_or(false)
                };
                if included {
                    let a = screen(x, y);
                    let b = screen(x + dx, y);
                    let c = screen(x + dx, y + dy);
                    let d = screen(x, y + dy);
                    let color = Color::from_rgba(55, 115, 220, 150);
                    draw_triangle(a, b, c, color);
                    draw_triangle(a, c, d, color);
                }
            }
        }
    }
    draw_text("2D plot  •  Esc closes", left, 25.0, 20.0, DARKGRAY);
}

fn draw_3d(plot: &Plot, orbit: &mut OrbitCamera) {
    let explicit_z = matches!(&plot.expression, smth::Expr::Equals(left, _) if left.len() == 1 && matches!(&left[0], smth::Expr::Variable(name) if name == "z"));
    if !explicit_z && plot.kind != PlotKind::Function {
        set_default_camera();
        draw_text("3D plots currently need z = f(x, y)", 24.0, 32.0, 22.0, RED);
        return;
    }
    if is_mouse_button_down(MouseButton::Left) {
        let delta = mouse_delta_position();
        orbit.yaw -= delta.x * 3.0;
        orbit.pitch = (orbit.pitch + delta.y * 3.0).clamp(0.08, 1.48);
    }
    let wheel = mouse_wheel().1;
    orbit.distance = (orbit.distance - wheel).clamp(2.0, 80.0);
    let horizontal = orbit.distance * orbit.pitch.cos();
    let camera = Camera3D {
        position: vec3(
            horizontal * orbit.yaw.cos(),
            horizontal * orbit.yaw.sin(),
            orbit.distance * orbit.pitch.sin(),
        ),
        target: vec3(0.0, 0.0, 0.0),
        up: vec3(0.0, 0.0, 1.0),
        ..Default::default()
    };
    set_camera(&camera);
    for i in -10..=10 {
        let v = i as f64;
        draw_line_3d(world(plot, v, -10.0, 0.0), world(plot, v, 10.0, 0.0), GRAY);
        draw_line_3d(world(plot, -10.0, v, 0.0), world(plot, 10.0, v, 0.0), GRAY);
    }
    draw_line_3d(
        world(plot, -10.0, 0.0, 0.0),
        world(plot, 10.0, 0.0, 0.0),
        RED,
    );
    draw_line_3d(
        world(plot, 0.0, -10.0, 0.0),
        world(plot, 0.0, 10.0, 0.0),
        GREEN,
    );
    draw_line_3d(
        world(plot, 0.0, 0.0, -10.0),
        world(plot, 0.0, 0.0, 10.0),
        BLUE,
    );
    let n = plot.resolution.clamp(8, 64);
    let (xmin, xmax) = plot.x_range;
    let (ymin, ymax) = plot.y_range;
    let mut rows: Vec<Vec<Option<Vec3>>> = vec![vec![None; n]; n];
    for (i, row) in rows.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate() {
            let x = xmin + (xmax - xmin) * i as f64 / (n - 1) as f64;
            let y = ymin + (ymax - ymin) * j as f64 / (n - 1) as f64;
            *slot = plot
                .evaluate(x, y, 0.0)
                .ok()
                .filter(|z| z.is_finite())
                .map(|z| world(plot, x, y, z));
        }
    }
    for i in 0..n {
        for j in 0..n {
            if let Some(a) = rows[i][j] {
                if i + 1 < n {
                    if let Some(b) = rows[i + 1][j] {
                        draw_line_3d(a, b, BLUE);
                    }
                }
                if j + 1 < n {
                    if let Some(b) = rows[i][j + 1] {
                        draw_line_3d(a, b, BLUE);
                    }
                }
            }
        }
    }
    set_default_camera();
    draw_text(
        "3D surface  |  drag: rotate  |  wheel: zoom  |  Esc: close",
        24.0,
        28.0,
        20.0,
        DARKGRAY,
    );
}

fn world(plot: &Plot, x: f64, y: f64, z: f64) -> Vec3 {
    let p = plot.transform(x, y, z);
    vec3(p[0] as f32, p[1] as f32, p[2] as f32)
}

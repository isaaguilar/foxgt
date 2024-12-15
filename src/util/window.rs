use crate::{WINDOW_X, WINDOW_Y};

use super::*;

#[derive(Resource, Debug)]
pub struct PixelScale(pub f32, pub f32);

#[derive(Component, Default)]
pub struct Scalers {
    pub left: Option<Val>,
    pub right: Option<Val>,
    pub top: Option<Val>,
    pub bottom: Option<Val>,
    pub width: Option<Val>,
}

pub fn hud_scale_updater(mut pixel_scale: ResMut<PixelScale>, windows: Query<&Window>) {
    let window_y = windows.single().resolution.size().y;
    let window_x = windows.single().resolution.size().x;
    // Maintain 16:9 aspect ratio calculations, even when not correct in window
    let normalized_x = 16.0 * window_y / 9.;
    let normalized_y = 9. * window_x / 16.0;
    pixel_scale.0 = windows.single().resolution.size().x / WINDOW_X;
    pixel_scale.1 = windows.single().resolution.size().y / WINDOW_Y;
}

pub fn hud_resizer(
    pixel_scale: Res<PixelScale>,
    mut resizeable_texts: Query<(&mut Transform, &mut Node, &Scalers)>,
) {
    let scale_x = pixel_scale.0;
    let scale_y = pixel_scale.1;
    for (mut t, mut s, scalers) in resizeable_texts.iter_mut() {
        t.scale = Vec3::splat(scale_x);

        match scalers.left {
            Some(val) => s.left = scale_val(scale_x, val),
            None => {}
        }
        match scalers.bottom {
            Some(val) => s.bottom = scale_val(scale_y, val),
            None => {}
        }
        match scalers.top {
            Some(val) => s.top = scale_val(scale_y, val),
            None => {}
        }
        match scalers.right {
            Some(val) => s.right = scale_val(scale_x, val),
            None => {}
        }
        match scalers.width {
            Some(val) => s.width = scale_val(scale_x, val),
            None => {}
        }
    }
}

fn scale_val(scale: f32, val: Val) -> Val {
    match val {
        Val::Auto => Val::Auto,
        Val::Px(n) => Val::Px(n * scale),
        Val::Percent(n) => Val::Percent(n),
        Val::Vw(_n) => todo!(),
        Val::Vh(_n) => todo!(),
        Val::VMin(_n) => todo!(),
        Val::VMax(_n) => todo!(),
    }
}

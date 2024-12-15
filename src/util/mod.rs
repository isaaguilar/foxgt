pub mod window;

use bevy::prelude::*;
use rand::Rng;

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn random_bool_one_in_n(n: u32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=n) == 1
}

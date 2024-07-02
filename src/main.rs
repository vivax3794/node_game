#![warn(
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::filetype_is_file,
    clippy::fn_to_numeric_cast_any,
    clippy::if_then_some_else_none,
    clippy::missing_const_for_fn,
    clippy::mixed_read_write_in_expression,
    clippy::panic,
    clippy::partial_pub_fields,
    clippy::same_name_method,
    clippy::str_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::try_err,
    clippy::unneeded_field_pattern,
    clippy::use_debug,
    clippy::verbose_file_reads,
    clippy::expect_used
)]
#![deny(
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::unimplemented,
    clippy::todo,
    clippy::dbg_macro,
    clippy::error_impl_error,
    clippy::exit,
    clippy::panic_in_result_fn,
    clippy::tests_outside_test_module
)]
#![allow(
    clippy::type_complexity,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

mod assets;
mod background;
mod gameplay;
mod node_editor;

#[allow(unused_imports)]
mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_debug_text_overlay::screen_print;
    pub use bevy_prototype_lyon::prelude::*;
    pub use bevy_rand::prelude::*;
    pub use bevy_states_utils::{AppExtension, Gc};
}
use bevy::input::common_conditions::input_just_pressed;
use prelude::*;

#[derive(States, Default, Clone, Hash, Eq, PartialEq, Debug)]
pub enum MainState {
    #[default]
    Loading,
    Playing,
}

#[derive(States, Default, Clone, Hash, Eq, PartialEq, Debug)]
pub enum PlayingState {
    #[default]
    None,
    ShootyTime,
    Editor,
}

#[repr(u32)]
enum ZIndex {
    Background,
    Bullet,
    Player,
    Cursor,
}

impl From<ZIndex> for f32 {
    fn from(value: ZIndex) -> Self {
        (value as u32) as f32 * 10.0
    }
}

fn main() {
    let mut app = App::new();
    #[cfg(feature = "release")]
    app.add_plugins(bevy_embedded_assets::EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
    });

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_debug_text_overlay::OverlayPlugin::default())
        .add_plugins(ShapePlugin)
        .add_plugins(bevy_rand::prelude::EntropyPlugin::<bevy_prng::WyRand>::default());
    app.insert_resource(ClearColor(Color::BLACK));

    #[cfg(feature = "dev")]
    {
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }

    app.init_state::<MainState>();
    app.init_state::<PlayingState>();
    app.add_substate(MainState::Playing, PlayingState::ShootyTime);
    app.add_gc_state::<MainState>();
    app.add_gc_state::<PlayingState>();

    app.add_plugins((
        assets::AssetPlugin,
        node_editor::NodeEditorPlugin,
        gameplay::GamePlayPlugin,
        background::BackgroundPlugin,
    ));

    app.add_systems(
        Update,
        (
            bevy::window::close_on_esc,
            swap_mode
                .run_if(in_state(MainState::Playing).and_then(input_just_pressed(KeyCode::KeyE))),
        ),
    );
    app.run();
}

fn swap_mode(state: Res<State<PlayingState>>, mut next: ResMut<NextState<PlayingState>>) {
    next.0 = Some(match **state {
        PlayingState::None => PlayingState::None,
        PlayingState::ShootyTime => PlayingState::Editor,
        PlayingState::Editor => PlayingState::ShootyTime,
    });
}

use bevy::prelude::*;

use crate::{
    colors::{CLEAR, DARK},
    states::AppState,
};

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), setup)
            .add_systems(Update, hover_help.run_if(in_state(AppState::Playing)));
    }
}

#[derive(Component)]
struct HelpButton;

#[derive(Component)]
struct HelpText;

const HELP_SYMBOL: &str = "?";
const HELP_TEXT: &str = "Left click and drag to move a piece.\n\n\
                        Right or middle click and drag to pan the camera.\n\n\
                        Press space to center the camera.";

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        // TODO padding
                        ..default()
                    },
                    background_color: CLEAR.into(),
                    ..default()
                })
                .insert(HelpButton)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            HELP_SYMBOL,
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: DARK,
                            },
                        ))
                        .insert(HelpText);
                });
        });
}

fn hover_help(
    interaction_query: Query<&Interaction>,
    mut text_query: Query<&mut Text, With<HelpText>>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    for interaction in &interaction_query {
        match interaction {
            Interaction::None => text.sections[0].value = String::from(HELP_SYMBOL),
            Interaction::Hovered => {
                text.sections[0].value = String::from(HELP_TEXT);
                // TODO text color
            }
            _ => (),
        }
    }
}

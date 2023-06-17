use std::{
    iter::{Cycle, Peekable},
    slice::Iter,
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
    colors::{DARK, LIGHTER},
    states::AppState,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadingMessage(String::from("Connecting to server")))
            .add_systems(OnEnter(AppState::Loading), load)
            .add_systems(
                Update,
                loading_animation
                    .run_if(in_state(AppState::Loading).or_else(in_state(AppState::Cutting)))
                    .run_if(on_timer(Duration::from_millis(150))),
            )
            .add_systems(
                Update,
                loading_display
                    .run_if(in_state(AppState::Loading).or_else(in_state(AppState::Cutting))),
            )
            .add_systems(OnEnter(AppState::Setup), setup)
            .add_systems(OnEnter(AppState::Playing), loading_done)
            .add_systems(Update, hover_help.run_if(in_state(AppState::Playing)))
            .add_systems(OnEnter(AppState::ConnectionLost), loading_done)
            .add_systems(OnEnter(AppState::ConnectionLost), connection_lost_message);
    }
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

#[derive(Component)]
struct LoadingNode;

#[derive(Component)]
struct LoadingText;

#[derive(Resource)]
struct LoadingTextCycle<'a>(Peekable<Cycle<Iter<'a, &'a str>>>);

#[derive(Resource, Default)]
pub struct LoadingMessage(pub String);

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(UiFont(font_handle.clone()));

    let cycle = [" . . .", " · . .", " . · .", " . . ·", " . . ."]
        .iter()
        .cycle()
        .peekable();
    commands.insert_resource(LoadingTextCycle(cycle));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(LoadingNode)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "",
                            TextStyle {
                                font: font_handle,
                                font_size: 25.0,
                                color: LIGHTER,
                            },
                        ))
                        .insert(LoadingText);
                });
        });
}

fn loading_animation(mut cycle: ResMut<LoadingTextCycle<'static>>) {
    let _ = *cycle.0.next().unwrap();
}

fn loading_display(
    mut text_query: Query<&mut Text, With<LoadingText>>,
    mut cycle: ResMut<LoadingTextCycle<'static>>,
    loading_msg: Res<LoadingMessage>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    let msg = format!("{}{}", loading_msg.0, *cycle.0.peek().unwrap());
    text.sections[0].value = msg;
}

fn loading_done(mut commands: Commands, loading_msg_query: Query<Entity, With<LoadingNode>>) {
    let loading_msg_entity = loading_msg_query.get_single().unwrap();
    if let Some(entity_commands) = commands.get_entity(loading_msg_entity) {
        entity_commands.despawn_recursive();
    }
}

#[derive(Component)]
struct HelpButton;

#[derive(Component)]
struct HelpText;

const HELP_SYMBOL: &str = "?";
const HELP_TEXT: &str = "• Left click and drag to move a piece\n\
                        • Right or middle click and drag to pan\n\
                        • Scroll to zoom\n\
                        • Press space to center the camera\n\n\
                        Made by Harrison Gieraltowski - harrisonmg.net";

fn setup(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn(NodeBundle {
            style: Style {
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
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .insert(HelpButton)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            HELP_SYMBOL,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 25.0,
                                color: LIGHTER,
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
            Interaction::Hovered => text.sections[0].value = String::from(HELP_TEXT),
            _ => (),
        }
    }
}

fn connection_lost_message(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Connection to sever lost. Try refreshing the page.",
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 25.0,
                            color: LIGHTER,
                        },
                    ));
                });
        });
}

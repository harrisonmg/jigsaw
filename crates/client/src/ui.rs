use std::{
    iter::{Cycle, Peekable},
    slice::Iter,
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer};
use game::Puzzle;
use gloo_file::{Blob, ObjectUrl};
use regex::RegexSetBuilder;
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;

use crate::{
    colors::{DARK, LIGHTER},
    states::AppState,
    util::despawn,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // setup
        app.insert_resource(LoadingMessage::default())
            .add_systems(Startup, startup)
            .add_systems(PostStartup, start_loading);

        // loading
        app.add_systems(
            Update,
            loading_display.run_if(not(in_state(AppState::Playing))),
        )
        .add_systems(
            Update,
            loading_animation
                .run_if(not(in_state(AppState::Playing)))
                .run_if(on_timer(Duration::from_millis(150))),
        );

        // playing
        app.add_systems(OnEnter(AppState::Playing), (enter_playing, stop_loading))
            .add_systems(OnExit(AppState::Playing), (exit_playing, start_loading))
            .add_systems(
                Update,
                (hover_help, hover_image_download).run_if(in_state(AppState::Playing)),
            );
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

#[derive(Resource)]
pub struct IsMobile(pub bool);

const MOBILE_WARNING: &str = "Warning:\n\n\
                              Interaction with the puzzle does not work on mobile browsers.\n\n\
                              Please try visiting with a desktop browser!";

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(UiFont(font_handle.clone()));

    let cycle = [" . . .", " · . .", " . · .", " . . ·", " . . ."]
        .iter()
        .cycle()
        .peekable();
    commands.insert_resource(LoadingTextCycle(cycle));

    let re = RegexSetBuilder::new([
    r"(android|bb\\d+|meego).+mobile|avantgo|bada\\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\\.(browser|link)|vodafone|wap|windows ce|xda|xiino",
    r"1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\\-(n|u)|c55\\/|capi|ccwa|cdm\\-|cell|chtm|cldc|cmd\\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\\-s|devi|dica|dmob|do(c|p)o|ds(12|\\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\\-|_)|g1 u|g560|gene|gf\\-5|g\\-mo|go(\\.w|od)|gr(ad|un)|haie|hcit|hd\\-(m|p|t)|hei\\-|hi(pt|ta)|hp( i|ip)|hs\\-c|ht(c(\\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\\-(20|go|ma)|i230|iac( |\\-|\\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\\/)|klon|kpt |kwc\\-|kyo(c|k)|le(no|xi)|lg( g|\\/(k|l|u)|50|54|\\-[a-w])|libw|lynx|m1\\-w|m3ga|m50\\/|ma(te|ui|xo)|mc(01|21|ca)|m\\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\\-2|po(ck|rt|se)|prox|psio|pt\\-g|qa\\-a|qc(07|12|21|32|60|\\-[2-7]|i\\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\\-|oo|p\\-)|sdk\\/|se(c(\\-|0|1)|47|mc|nd|ri)|sgh\\-|shar|sie(\\-|m)|sk\\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\\-|v\\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\\-|tdg\\-|tel(i|m)|tim\\-|t\\-mo|to(pl|sh)|ts(70|m\\-|m3|m5)|tx\\-9|up(\\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\\-|your|zeto|zte\\-",
    ])
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .unwrap();

    let window = web_sys::window().unwrap();
    let user_agent = window.navigator().user_agent().unwrap();
    let is_mobile = IsMobile(re.is_match(&user_agent));

    if is_mobile.0 {
        commands
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Start,
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
                            margin: UiRect::all(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        background_color: DARK.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                MOBILE_WARNING,
                                TextStyle {
                                    font: font_handle,
                                    font_size: 25.0,
                                    color: LIGHTER,
                                },
                            )
                            .with_text_alignment(TextAlignment::Center),
                        );
                    });
            });
    }

    commands.insert_resource(is_mobile);
}

fn start_loading(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
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
                                font: font.0.clone(),
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

fn stop_loading(mut commands: Commands, loading_msg_query: Query<Entity, With<LoadingNode>>) {
    despawn(loading_msg_query, &mut commands);
}

#[derive(Component)]
struct HelpNode;

#[derive(Component)]
struct HelpButton;

#[derive(Component)]
struct HelpText;

const HELP_SYMBOL: &str = "?";
const HELP_TEXT: &str = "• Left click to grab and place pieces, or click and drag to move them\n\
                        • Right or middle click and drag to pan\n\
                        • Scroll to zoom\n\
                        • Press space to center the camera\n\n\
                        Made by Harrison Gieraltowski - harrisonmg.net";

#[derive(Component)]
struct ImageDownloadNode;

#[derive(Component)]
struct ImageDownloadButton;

#[derive(Component)]
struct ImageDownloadText;

const IMAGE_DOWNLOAD_SYMBOL: &str = "↓";
const IMAGE_DOWNLOAD_TEXT: &str = "Click to download the full puzzle image";

const BUTTON_SIZE: Val = Val::Px(30.0);

fn enter_playing(mut commands: Commands, font: Res<UiFont>, is_mobile: Res<IsMobile>) {
    if is_mobile.0 {
        return;
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..default()
            },
            ..default()
        })
        .insert(HelpNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: BUTTON_SIZE,
                        height: BUTTON_SIZE,
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

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::End,
                ..default()
            },
            ..default()
        })
        .insert(ImageDownloadNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: BUTTON_SIZE,
                        height: BUTTON_SIZE,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .insert(ImageDownloadButton)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            IMAGE_DOWNLOAD_SYMBOL,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 25.0,
                                color: LIGHTER,
                            },
                        ))
                        .insert(ImageDownloadText);
                });
        });
}

fn exit_playing(
    mut commands: Commands,
    help_node_query: Query<Entity, With<HelpNode>>,
    image_download_node_query: Query<Entity, With<ImageDownloadNode>>,
) {
    despawn(help_node_query, &mut commands);
    despawn(image_download_node_query, &mut commands);
}

#[allow(clippy::type_complexity)]
fn hover_help(
    mut interaction_query: Query<
        (&Interaction, &mut Style),
        (Changed<Interaction>, With<HelpButton>),
    >,
    mut text_query: Query<&mut Text, With<HelpText>>,
    puzzle: Res<Puzzle>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    for (interaction, mut style) in &mut interaction_query {
        match interaction {
            Interaction::None => {
                text.sections[0].value = String::from(HELP_SYMBOL);
                style.width = BUTTON_SIZE;
                style.height = BUTTON_SIZE;
            }
            Interaction::Hovered => {
                text.sections[0].value = String::from(HELP_TEXT);
                style.width = Val::Auto;
                style.height = Val::Auto;
            }
            Interaction::Clicked => {
                info!("{:#?}", puzzle);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn hover_image_download(
    mut interaction_query: Query<
        (&Interaction, &mut Style),
        (Changed<Interaction>, With<ImageDownloadButton>),
    >,
    mut text_query: Query<&mut Text, With<ImageDownloadText>>,
    puzzle: Res<Puzzle>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    for (interaction, mut style) in &mut interaction_query {
        match interaction {
            Interaction::None => {
                text.sections[0].value = String::from(IMAGE_DOWNLOAD_SYMBOL);
                style.width = BUTTON_SIZE;
                style.height = BUTTON_SIZE;
            }
            Interaction::Hovered => {
                text.sections[0].value = String::from(IMAGE_DOWNLOAD_TEXT);
                style.width = Val::Auto;
                style.height = Val::Auto;
            }
            Interaction::Clicked => {
                let bytes = puzzle.raw_image();
                let blob = Blob::new(bytes.as_ref());
                let object_url = ObjectUrl::from(blob);

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                let link = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<HtmlAnchorElement>()
                    .unwrap();
                link.style().set_property("display", "none").unwrap();
                link.set_href(object_url.as_ref());
                link.set_download("cheater.png");

                let body = document.body().unwrap();
                body.append_child(&link).unwrap();
                link.click();
            }
        }
    }
}

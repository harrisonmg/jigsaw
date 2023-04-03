use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

#[derive(Resource)]
struct OnResizeSender(Mutex<Sender<()>>);
#[derive(Resource)]
struct OnResizeReceiver(Mutex<Receiver<()>>);

pub struct FullViewportPlugin;

impl Plugin for FullViewportPlugin {
    fn build(&self, app: &mut App) {
        let channel = std::sync::mpsc::channel();
        let resize_sender = OnResizeSender(Mutex::new(channel.0));
        let resize_receiver = OnResizeReceiver(Mutex::new(channel.1));

        app.insert_resource(resize_sender)
            .insert_resource(resize_receiver)
            .add_systems(Startup, setup_viewport_resize_system)
            .add_systems(Update, viewport_resize_system);
    }
}

pub fn get_viewport_size() -> (f32, f32) {
    let window = web_sys::window().unwrap();
    let document_element = window.document().unwrap().document_element().unwrap();

    let width = document_element.client_width();
    let height = document_element.client_height();

    (width as f32, height as f32)
}

fn setup_viewport_resize_system(resize_sender: Res<OnResizeSender>) {
    let web_window = web_sys::window().unwrap();
    let local_sender = resize_sender.0.lock().unwrap().clone();

    local_sender.send(()).unwrap();

    gloo_events::EventListener::new(&web_window, "resize", move |_event| {
        local_sender.send(()).unwrap();
    })
    .forget();
}

fn viewport_resize_system(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    resize_receiver: Res<OnResizeReceiver>,
) {
    if resize_receiver.0.lock().unwrap().try_recv().is_ok() {
        if let Ok(mut window) = window_query.get_single_mut() {
            let size = get_viewport_size();
            window.resolution = WindowResolution::new(size.0, size.1);
        }
    }
}

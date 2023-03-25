use bevy::prelude::*;
use gloo_events::{EventListener, EventListenerOptions};
use wasm_bindgen::prelude::JsCast;
use web_sys::HtmlCanvasElement;

pub struct DisableContextMenuPlugin;

impl Plugin for DisableContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, disable_context_menu);
    }
}

fn disable_context_menu() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let collection = document.get_elements_by_tag_name("canvas");
    for i in 0..collection.length() {
        let canvas = collection
            .item(i)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let options = EventListenerOptions::enable_prevent_default();
        let event_listener =
            EventListener::new_with_options(&canvas, "contextmenu", options, |event| {
                event.prevent_default();
                event.stop_propagation();
            });
        event_listener.forget();
    }
}

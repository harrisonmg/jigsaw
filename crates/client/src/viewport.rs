pub fn get_viewport_size() -> (f32, f32) {
    let window = web_sys::window().unwrap();
    let document_element = window.document().unwrap().document_element().unwrap();

    let width = document_element.client_width();
    let height = document_element.client_height();

    (width as f32, height as f32)
}


extern crate glfw;
extern crate regl;
extern crate cgmath;

use glfw::{Action, Context, Key, MouseButton};

mod graphics;

struct AppState {
    graphics: graphics::Graphics,
    tracked_position: Option<(f64, f64)>,
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(300, 300, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    // window.set_cursor_enter_polling(true);
    window.set_mouse_button_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    regl::load_with(|s| window.get_proc_address(s));

    let mut app_state = AppState {
        graphics: graphics::Graphics::new(300, 300),
        tracked_position: None,
    };

    while !window.should_close() {
        glfw.wait_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut app_state);
        }

        app_state.graphics.draw();

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, app_state: &mut AppState) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        },
        glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
            app_state.tracked_position = Some(window.get_cursor_pos());
        },
        glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
            app_state.tracked_position = None;
        },
        glfw::WindowEvent::CursorPos(x, y) => {
            if let Some((prev_x, prev_y)) = app_state.tracked_position {
                let (delta_x, delta_y) = (prev_x - x, prev_y - y);
                app_state.graphics.camera_orientation(delta_x, delta_y);
                app_state.tracked_position = Some((x, y));
            }
        },
        glfw::WindowEvent::FramebufferSize(width, height) => {
            app_state.graphics.viewport_size(width, height);
        },
        glfw::WindowEvent::Scroll(_, vertical_scroll) => {
            app_state.graphics.camera_distance(-vertical_scroll);
        }
        _ => {}
    }
}

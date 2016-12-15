//! A renderer using SDL2.

use dijkstra_map::*;
use mobiles::*;
use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::collections::BTreeMap;
use types::*;
use ui::UI;

/// A user interface using SDL2.
#[allow(missing_debug_implementations,missing_copy_implementations)]
pub struct SdlUI {
    /// The main interface to SDL.
    context: sdl2::Sdl,
    /// The event pump: used to wait for and gather input events.
    events: sdl2::EventPump,
    /// The video subsystem.
    video: sdl2::VideoSubsystem,
    /// The renderer
    renderer: sdl2::render::Renderer<'static>,
}

/// Construct a new SDL2 interface. Should only be called once.
pub fn new() -> Result<SdlUI, String> {
    let context = try!(sdl2::init());
    let events = try!(context.event_pump());
    let video = try!(context.video());
    let window =
        match video.window("Rogue Mayor", 1024, 768).position_centered().opengl().build() {
            Ok(win) => win,
            Err(e) => return Err(format!("{}", e)),
        };
    let renderer = match window.renderer().build() {
        Ok(ren) => ren,
        Err(e) => return Err(format!("{}", e)),
    };

    Ok(SdlUI {
        context: context,
        events: events,
        video: video,
        renderer: renderer,
    })
}

impl UI for SdlUI {
    fn render(&mut self, mobs: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();
        self.renderer.present();
    }

    fn input(&mut self) -> Command {
        match self.events.wait_event() {
            Event::Quit { .. } |
            Event::AppTerminating { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => Command::Quit,
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => Command::Skip,
            _ => self.input(), // Ignore unexpected input.
        }
    }
}

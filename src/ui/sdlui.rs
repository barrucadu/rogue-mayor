//! A renderer using SDL2.

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::*;
use sdl2;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use statics::*;
use std::collections::BTreeMap;
use std::f64;
use std::path::Path;
use types::*;
use ui::UI;

// The font
const FONT_PATH: &'static str = "font/Anonymous Pro.ttf";
const FONT_SIZE: u16 = 24;

// Size of the visible viewport, in grid cells.
const VIEWPORT_CELL_HEIGHT: usize = 100;
const VIEWPORT_CELL_WIDTH: usize = 200;

// Size of a cell, in picels.
const CELL_PIXEL_HEIGHT: usize = 8;
const CELL_PIXEL_WIDTH: usize = 8;

// Size of margins around viewport.
const MARGIN_PIXEL_TOP: usize = 80;
const MARGIN_PIXEL_LEFT: usize = 0;
const MARGIN_PIXEL_BOTTOM: usize = 0;
const MARGIN_PIXEL_RIGHT: usize = 0;

// Number of log entries to fit in the top margin.
const LOG_ENTRIES_VISIBLE: usize = 3;

// Some helpful derived stuff
const SCREEN_WIDTH: u32 = (MARGIN_PIXEL_LEFT + MARGIN_PIXEL_BOTTOM +
                           CELL_PIXEL_WIDTH * VIEWPORT_CELL_WIDTH) as u32;
const SCREEN_HEIGHT: u32 =
    (MARGIN_PIXEL_TOP + MARGIN_PIXEL_BOTTOM + CELL_PIXEL_HEIGHT * VIEWPORT_CELL_HEIGHT) as u32;
const CONTENT_WIDTH: u32 = SCREEN_WIDTH - MARGIN_PIXEL_LEFT as u32 - MARGIN_PIXEL_RIGHT as u32;
const CONTENT_HEIGHT: u32 = SCREEN_HEIGHT - MARGIN_PIXEL_TOP as u32 - MARGIN_PIXEL_BOTTOM as u32;
const LOG_ENTRY_HEIGHT: u32 = (MARGIN_PIXEL_TOP / LOG_ENTRIES_VISIBLE) as u32;

/// A user interface using SDL2.
#[allow(missing_debug_implementations,missing_copy_implementations)]
pub struct SdlUI {
    /// The top-left of the viewport.
    viewport: Point,
    /// The main interface to SDL.
    context: Sdl,
    /// The event pump: used to wait for and gather input events.
    events: EventPump,
    /// The video subsystem.
    video: VideoSubsystem,
    /// The renderer
    renderer: Renderer<'static>,
    /// The TTF context.
    ttf: Sdl2TtfContext,
    /// Debugging display: whether to show heatmaps.
    show_heatmap: bool,
    /// Debugging display: the heatmap to render.
    active_heatmap: (Style, MapTag),
}

/// Debugging display: a heatmap style to render.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Style {
    /// Render the approach map.
    Approach,
    /// Render the cowardly fleeing map.
    FleeCowardly,
    /// Render the bravely fleeing map.
    FleeBravely,
}

impl UI for SdlUI {
    fn render(&mut self, mobs: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();

        // Render the message log.
        self.render_log(world);

        // Render the world OR heatmap.
        self.render_world(mobs, maps, world);

        // Finally, display everything.
        self.renderer.present();
    }

    fn input(&mut self) -> Command {
        match self.events.wait_event() {
            Event::Quit { .. } |
            Event::AppTerminating { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => Command::Quit,
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => Command::Skip,
            Event::KeyDown { keycode: Some(Keycode::F5), .. } => Command::Render,
            Event::KeyDown { keycode: Some(Keycode::Tab), .. } => {
                self.active_heatmap = next_heatmap(self.active_heatmap);
                println!("DEBUG: rendering heatmap {:?}", self.active_heatmap);
                Command::Render
            }
            Event::KeyDown { keycode: Some(Keycode::Semicolon), .. } => {
                self.show_heatmap = !self.show_heatmap;
                println!("DEBUG: toggling heatmap to {:?}", self.show_heatmap);
                Command::Render
            }
            _ => self.input(), // Ignore unexpected input.
        }
    }
}

impl SdlUI {
    /// Construct a new SDL2 interface. Should only be called once.
    pub fn new() -> Result<SdlUI, String> {
        let context = try!(sdl2::init());
        let events = try!(context.event_pump());
        let video = try!(context.video());
        let window = match video.window("Rogue Mayor", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build() {
            Ok(win) => win,
            Err(e) => return Err(format!("{}", e)),
        };
        let renderer = match window.renderer().build() {
            Ok(ren) => ren,
            Err(e) => return Err(format!("{}", e)),
        };
        let ttf = match sdl2::ttf::init() {
            Ok(ttf) => ttf,
            Err(e) => return Err(format!("{}", e)),
        };

        Ok(SdlUI {
            viewport: Point { x: 0, y: 0 },
            context: context,
            events: events,
            video: video,
            renderer: renderer,
            ttf: ttf,
            show_heatmap: false,
            active_heatmap: (Style::Approach, MapTag::Adventure),
        })
    }

    /// Render the message log.
    fn render_log(&mut self, world: &World) {
        let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();

        let color = Color::RGB(255, 255, 255);
        let mut done = 0;
        for msg in world.messages.iter().take(LOG_ENTRIES_VISIBLE) {
            let bbox = Rect::new(
                MARGIN_PIXEL_LEFT as i32,
                ((LOG_ENTRIES_VISIBLE - done - 1) as u32 * LOG_ENTRY_HEIGHT) as i32,
                CONTENT_WIDTH,
                LOG_ENTRY_HEIGHT
            );
            let surface = font.render(msg.msg.as_str()).blended(color).unwrap();
            let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
            render_in(&mut self.renderer, &mut texture, bbox, false, true);
            done += 1;
        }
    }

    /// Render the world, with a heatmap overlay if enabled.
    fn render_world(&mut self, mobs: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();

        // Render the active heatmap.
        let (style, tag) = self.active_heatmap;
        let heatmap = maps.get(tag);
        let map = match style {
            Style::Approach => &heatmap.approach,
            Style::FleeCowardly => &heatmap.flee_cowardly,
            Style::FleeBravely => &heatmap.flee_bravely,
        };

        // Find the min and max values in the heatmap.
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        if self.show_heatmap {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let val = map.at(Point { x: x, y: y });
                    if val > max && val != f64::MAX {
                        max = val;
                    }
                    if val < min {
                        min = val;
                    }
                }
            }
        }

        // Render every cell.
        for dy in 0..VIEWPORT_CELL_HEIGHT {
            if self.viewport.y + dy >= HEIGHT {
                break;
            }
            for dx in 0..VIEWPORT_CELL_WIDTH {
                if self.viewport.x + dx >= WIDTH {
                    break;
                }
                let here = Point {
                    x: self.viewport.x + dx,
                    y: self.viewport.y + dy,
                };
                let color = if self.show_heatmap {
                    let val = map.at(here);
                    let p = if val == f64::MAX {
                        1.0
                    } else {
                        (val - min) / (max - min)
                    };
                    Color::RGB((255.0 * p).round() as u8,
                               (255.0 * (1.0 - p)).round() as u8,
                               0)
                } else {
                    Color::RGB(0, 0, 0)
                };
                render_cell(&mut self.renderer,
                            &font,
                            self.viewport.x + dx,
                            self.viewport.y + dy,
                            world.statics.at(here),
                            mobs.get(&here),
                            color);
            }
        }
    }
}

/// Advance to the next active heatmap, or turn it off on the last one.
fn next_heatmap(heatmap: (Style, MapTag)) -> (Style, MapTag) {
    match heatmap {
        (Style::Approach, tag) => (Style::FleeCowardly, tag),
        (Style::FleeCowardly, tag) => (Style::FleeBravely, tag),
        (Style::FleeBravely, MapTag::Adventure) => (Style::Approach, MapTag::GeneralStore),
        (Style::FleeBravely, MapTag::GeneralStore) => (Style::Approach, MapTag::Rest),
        (Style::FleeBravely, MapTag::Rest) => (Style::Approach, MapTag::Sustenance),
        (Style::FleeBravely, MapTag::Sustenance) => (Style::Approach, MapTag::Adventure),
    }
}

/// Render a cell.
fn render_cell(renderer: &mut Renderer<'static>,
               font: &Font,
               cell_x: usize,
               cell_y: usize,
               s: Option<Static>,
               m: Option<&Mobile>,
               color: Color) {
    let x = MARGIN_PIXEL_LEFT + cell_x * CELL_PIXEL_WIDTH;
    let y = MARGIN_PIXEL_TOP + cell_y * CELL_PIXEL_HEIGHT;
    let rect = Rect::new(x as i32,
                         y as i32,
                         CELL_PIXEL_WIDTH as u32,
                         CELL_PIXEL_HEIGHT as u32);
    renderer.set_draw_color(color);
    let _ = renderer.fill_rect(rect);

    match (m, s) {
        (Some(mob), _) => render_mobile(renderer, font, rect, mob),
        (_, Some(stat)) => render_static(renderer, font, rect, &stat),
        _ => {}
    }
}

/// Render a static.
fn render_static(renderer: &mut Renderer<'static>, font: &Font, rect: Rect, s: &Static) {
    let (ch, foreground, background) = match *s {
        Static::Wall => ('#', Color::RGB(255,255,255), Some(Color::RGB(133,94,66))), // "white" on "dark wood"
        Static::Door => ('║', Color::RGB(0,0,0), Some(Color::RGB(133,94,66))), // "black" on "dark wood"
    };
    render_occupant(renderer, font, rect, ch, foreground, background)
}

/// Render a mobile.
fn render_mobile(_: &mut Renderer<'static>, _: &Font, _: Rect, _: &Mobile) {
    // They don't exist yet!
}

/// Render an occupant of a cell.
fn render_occupant(renderer: &mut Renderer<'static>, font: &Font, rect: Rect, ch: char, foreground: Color, background: Option<Color>) {
    if let Some(bg) = background {
        renderer.set_draw_color(bg);
        let _ = renderer.fill_rect(rect);
    }

    let surface = font.render(ch.to_string().as_str()).blended(foreground).unwrap();
    let mut texture = renderer.create_texture_from_surface(&surface).unwrap();
    render_in(renderer, &mut texture, rect, true, true);
}

/// Copy a texture into a bounding box, scaling it if necessary.
fn render_in(renderer: &mut Renderer<'static>, texture: &mut Texture, bbox: Rect, center_horiz:bool, center_vert:bool) {
    let TextureQuery { width, height, .. } = texture.query();

    // The target to render into.
    let mut target = Rect::new(bbox.x(), bbox.y(), width, height);

    // Scale down.
    let wr = width as f32 / bbox.width() as f32;
    let hr = height as f32 / bbox.height() as f32;
    if width > bbox.width() || height > bbox.height() {
        println!("WARN: scaling texture down, this will look worse!");
        if width > height {
            target.set_width(bbox.width());
            target.set_height((height as f32 / wr) as u32);
        } else {
            target.set_width((width as f32 / hr) as u32);
            target.set_height(bbox.height());
        }
    }

    // Center horizontally.
    if center_horiz && target.width() < bbox.width() {
        let x = target.x();
        let w = target.width();
        target.set_x(x + (bbox.width() - w) as i32 / 2);
    }

    // Center vertically.
    if center_vert && target.height() < bbox.height() {
        let y = target.y();
        let h = target.height();
        target.set_y(y + (bbox.height() - h) as i32 / 2);
    }

    // Render the texture in the target rect.
    let _ = renderer.copy(texture, None, Some(target));
}

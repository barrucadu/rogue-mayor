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
use sdl2::render::{Renderer, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
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
    fn render(&mut self, _: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();

        // Render the message log.
        self.render_log(world);

        // Render the world.
        self.render_world(world);

        // Overlay the active heatmap as half opacity.
        if self.show_heatmap {
            self.render_heatmap(maps, 127);
        }

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

        let mut done = 0;
        for msg in world.messages.iter().take(LOG_ENTRIES_VISIBLE) {
            render_text(&mut self.renderer,
                        &font,
                        msg.msg.clone(),
                        MARGIN_PIXEL_LEFT as i32,
                        ((LOG_ENTRIES_VISIBLE - done - 1) as u32 * LOG_ENTRY_HEIGHT) as i32,
                        Color::RGB(255, 255, 255));
            done += 1;
        }
    }

    /// Render the active heatmap with the given alpha level.
    fn render_heatmap(&mut self, maps: &Maps, alpha: u8) {
        let (style, tag) = self.active_heatmap;

        // Render the active heatmap.
        let heatmap = maps.get(tag);
        let map = match style {
            Style::Approach => &heatmap.approach,
            Style::FleeCowardly => &heatmap.flee_cowardly,
            Style::FleeBravely => &heatmap.flee_bravely,
        };

        // Find the min and max values in the heatmap.
        let mut min = f64::MAX;
        let mut max = f64::MIN;
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

        // Render every cell.
        for dy in 0..VIEWPORT_CELL_HEIGHT {
            if self.viewport.y + dy >= HEIGHT {
                break;
            }
            for dx in 0..VIEWPORT_CELL_WIDTH {
                if self.viewport.x + dx >= WIDTH {
                    break;
                }
                let val = map.at(Point {
                    x: self.viewport.x + dx,
                    y: self.viewport.y + dy,
                });
                let p = if val == f64::MAX {
                    1.0
                } else {
                    (val - min) / (max - min)
                };
                render_cell(&mut self.renderer,
                            self.viewport.x + dx,
                            self.viewport.y + dy,
                            Color::RGBA((255.0 * p).round() as u8,
                                        (255.0 * (1.0 - p)).round() as u8,
                                        0,
                                        alpha));
            }
        }
    }


    /// Render the world.
    fn render_world(&mut self, world: &World) {
        for dy in 0..VIEWPORT_CELL_HEIGHT {
            if self.viewport.y + dy > HEIGHT {
                break;
            }
            for dx in 0..VIEWPORT_CELL_WIDTH {
                if self.viewport.x + dx > WIDTH {
                    break;
                }
                let color = if world.occupied.at(Point {
                    x: self.viewport.x + dx,
                    y: self.viewport.y + dy,
                }) {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(0, 0, 0)
                };
                render_cell(&mut self.renderer,
                            self.viewport.x + dx,
                            self.viewport.y + dy,
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

/// Render some text.
fn render_text(renderer: &mut Renderer<'static>,
               font: &Font,
               text: String,
               left: i32,
               top: i32,
               color: Color) {
    let surface = font.render(text.as_str()).blended(color).unwrap();
    let mut texture = renderer.create_texture_from_surface(&surface).unwrap();
    let TextureQuery { width, height, .. } = texture.query();
    let rect = Rect::new(left, top, width, height);
    let _ = renderer.copy(&mut texture, None, Some(rect));
}

/// Render a cell.
fn render_cell(renderer: &mut Renderer<'static>, cell_x: usize, cell_y: usize, color: Color) {
    let x = MARGIN_PIXEL_LEFT + cell_x * CELL_PIXEL_WIDTH;
    let y = MARGIN_PIXEL_TOP + cell_y * CELL_PIXEL_HEIGHT;
    let rect = Rect::new(x as i32,
                         y as i32,
                         CELL_PIXEL_WIDTH as u32,
                         CELL_PIXEL_HEIGHT as u32);
    renderer.set_draw_color(color);
    let _ = renderer.fill_rect(rect);
}

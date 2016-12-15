//! A renderer using SDL2.

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::*;
use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::BTreeMap;
use std::f64;
use types::*;
use ui::UI;

// Size of the visible viewport, in grid cells.
const VIEWPORT_CELL_HEIGHT: usize = 100;
const VIEWPORT_CELL_WIDTH: usize = 200;

// Size of a cell, in picels.
const CELL_PIXEL_HEIGHT: usize = 8;
const CELL_PIXEL_WIDTH: usize = 8;

// Size of margins around viewport.
const MARGIN_PIXEL_TOP: usize = 0;
const MARGIN_PIXEL_LEFT: usize = 0;
const MARGIN_PIXEL_BOTTOM: usize = 0;
const MARGIN_PIXEL_RIGHT: usize = 0;

/// A user interface using SDL2.
#[allow(missing_debug_implementations,missing_copy_implementations)]
pub struct SdlUI {
    /// The top-left of the viewport.
    viewport: Point,
    /// The main interface to SDL.
    context: sdl2::Sdl,
    /// The event pump: used to wait for and gather input events.
    events: sdl2::EventPump,
    /// The video subsystem.
    video: sdl2::VideoSubsystem,
    /// The renderer
    renderer: sdl2::render::Renderer<'static>,
    /// Debugging display: the heatmap to render.
    active_heatmap: Option<(Style, MapTag)>,
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

impl SdlUI {
    /// Construct a new SDL2 interface. Should only be called once.
    pub fn new() -> Result<SdlUI, String> {
        let context = try!(sdl2::init());
        let events = try!(context.event_pump());
        let video = try!(context.video());
        let window = match video.window("Rogue Mayor",
                    (MARGIN_PIXEL_LEFT + MARGIN_PIXEL_BOTTOM +
                     CELL_PIXEL_WIDTH * VIEWPORT_CELL_WIDTH) as u32,
                    (MARGIN_PIXEL_TOP + MARGIN_PIXEL_BOTTOM +
                     CELL_PIXEL_HEIGHT * VIEWPORT_CELL_HEIGHT) as u32)
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

        Ok(SdlUI {
            viewport: Point { x: 0, y: 0 },
            context: context,
            events: events,
            video: video,
            renderer: renderer,
            active_heatmap: None,
        })
    }

    /// Advance to the next active heatmap, or turn it off on the last one.
    fn next_heatmap(&mut self) {
        self.active_heatmap = match self.active_heatmap {
            None => Some((Style::Approach, MapTag::Adventure)),
            Some((Style::Approach, tag)) => Some((Style::FleeCowardly, tag)),
            Some((Style::FleeCowardly, tag)) => Some((Style::FleeBravely, tag)),
            Some((Style::FleeBravely, MapTag::Adventure)) => {
                Some((Style::Approach, MapTag::GeneralStore))
            }
            Some((Style::FleeBravely, MapTag::GeneralStore)) => {
                Some((Style::Approach, MapTag::Rest))
            }
            Some((Style::FleeBravely, MapTag::Rest)) => Some((Style::Approach, MapTag::Sustenance)),
            Some((Style::FleeBravely, MapTag::Sustenance)) => None,
        }
    }
}

impl UI for SdlUI {
    fn render(&mut self, _: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();

        match self.active_heatmap {
            Some((style, tag)) => {
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

                for dy in 0..VIEWPORT_CELL_HEIGHT {
                    if self.viewport.y + dy >= HEIGHT {
                        break;
                    }
                    for dx in 0..VIEWPORT_CELL_WIDTH {
                        if self.viewport.x + dx >= WIDTH {
                            break;
                        }
                        let x = MARGIN_PIXEL_LEFT + (self.viewport.x + dx) * CELL_PIXEL_WIDTH;
                        let y = MARGIN_PIXEL_TOP + (self.viewport.y + dy) * CELL_PIXEL_HEIGHT;
                        let rect = Rect::new(x as i32,
                                             y as i32,
                                             CELL_PIXEL_WIDTH as u32,
                                             CELL_PIXEL_HEIGHT as u32);
                        let val = map.at(Point {
                            x: self.viewport.x + dx,
                            y: self.viewport.y + dy,
                        });
                        let p = if val == f64::MAX {
                            1.0
                        } else {
                            (val - min) / (max - min)
                        };
                        self.renderer.set_draw_color(Color::RGB((255.0 * p).round() as u8,
                                                                (255.0 * (1.0 - p)).round() as u8,
                                                                0));
                        let _ = self.renderer.fill_rect(rect);
                    }
                }
            }
            None => {
                // Render the active occupied cells.
                for dy in 0..VIEWPORT_CELL_HEIGHT {
                    if self.viewport.y + dy > HEIGHT {
                        break;
                    }
                    for dx in 0..VIEWPORT_CELL_WIDTH {
                        if self.viewport.x + dx > WIDTH {
                            break;
                        }
                        self.renderer.set_draw_color(if world.occupied.at(Point {
                            x: self.viewport.x + dx,
                            y: self.viewport.y + dy,
                        }) {
                            Color::RGB(255, 255, 255)
                        } else {
                            Color::RGB(0, 0, 0)
                        });
                        let x = MARGIN_PIXEL_LEFT + (self.viewport.x + dx) * CELL_PIXEL_WIDTH;
                        let y = MARGIN_PIXEL_TOP + (self.viewport.y + dy) * CELL_PIXEL_HEIGHT;
                        let rect = Rect::new(x as i32,
                                             y as i32,
                                             CELL_PIXEL_WIDTH as u32,
                                             CELL_PIXEL_HEIGHT as u32);
                        let _ = self.renderer.fill_rect(rect);
                    }
                }
            }
        }

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
                self.next_heatmap();
                println!("DEBUG: rendering heatmap {:?}", self.active_heatmap);
                Command::Render
            }
            _ => self.input(), // Ignore unexpected input.
        }
    }
}

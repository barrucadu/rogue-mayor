//! A renderer using SDL2.

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::*;
use sdl2;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use statics::*;
use std::cmp::min;
use std::collections::BTreeMap;
use std::f64;
use std::path::Path;
use types::*;
use ui::UI;

// The font
const FONT_PATH: &'static str = "FSEX300.ttf";
const FONT_SIZE: u16 = 12;

// Size of the visible viewport, in grid cells.
const VIEWPORT_CELL_HEIGHT: usize = 50;
const VIEWPORT_CELL_WIDTH: usize = 100;

// Everything is done in terms of rows and columns, which are made of
// fixed-size cells.
const CELL_PIXEL_HEIGHT: usize = 16;
const CELL_PIXEL_WIDTH: usize = 16;

// Number of rows gap between log entries and the map.
const LOG_GAP: usize = 2;

// Number of log entries to display above the map.
const LOG_ENTRIES_VISIBLE: usize = 7;

// Number of cells to "overshoot" the cursor by when scrolling.
const SCROLL_OVERSHOOT: usize = 25;

// Some helpful derived stuff
const SCREEN_WIDTH: u32 = (CELL_PIXEL_WIDTH * VIEWPORT_CELL_WIDTH) as u32;
const SCREEN_HEIGHT: u32 =
    (CELL_PIXEL_HEIGHT * (LOG_ENTRIES_VISIBLE + LOG_GAP + VIEWPORT_CELL_HEIGHT)) as u32;

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
    /// Whether the cursor is being moved by the mouse or not.
    is_mousing_cursor: bool,
    /// Whether we're zooming around or not (shift held down).
    is_zooming_cursor: bool,
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
    fn initial_cursor() -> Point {
        Point {
            x: VIEWPORT_CELL_WIDTH / 2,
            y: VIEWPORT_CELL_HEIGHT / 2,
        }
    }

    fn render(&mut self, mobs: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();

        // Scroll the viewport if the cursor is outside of it.
        if world.cursor.x < self.viewport.x {
            self.viewport.x = world.cursor.x.saturating_sub(SCROLL_OVERSHOOT);
        }
        if world.cursor.x > self.viewport.x + VIEWPORT_CELL_WIDTH {
            self.viewport.x = min(WIDTH - VIEWPORT_CELL_WIDTH,
                                  world.cursor.x + SCROLL_OVERSHOOT - VIEWPORT_CELL_WIDTH)
        }
        if world.cursor.y < self.viewport.y {
            self.viewport.y = world.cursor.y.saturating_sub(SCROLL_OVERSHOOT);
        }
        if world.cursor.y > self.viewport.y + VIEWPORT_CELL_HEIGHT {
            self.viewport.y = min(HEIGHT - VIEWPORT_CELL_HEIGHT,
                                  world.cursor.y + SCROLL_OVERSHOOT - VIEWPORT_CELL_HEIGHT)
        }

        // Render the message log.
        self.render_log(world);

        // Render the world OR heatmap.
        self.render_world(mobs, maps, world);

        // Display the cursor on top of everything else.
        self.render_cursor(world.cursor);

        // Finally, display everything.
        self.renderer.present();
    }

    fn input(&mut self, cursor: Point) -> Command {
        match self.events.wait_event() {
            // Exit
            Event::Quit { .. } |
            Event::AppTerminating { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => Command::Quit,
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => Command::Skip,

            // Cursor
            Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                self.is_mousing_cursor = !self.is_mousing_cursor;
                if self.is_mousing_cursor {
                    Command::SetCursorTo(cursor_from_mouse(self.viewport, x, y))
                } else {
                    Command::Render
                }
            }
            Event::KeyDown { keycode: Some(Keycode::LShift), .. } |
            Event::KeyDown { keycode: Some(Keycode::RShift), .. } => {
                self.is_zooming_cursor = true;
                self.input(cursor)
            }
            Event::KeyUp { keycode: Some(Keycode::LShift), .. } |
            Event::KeyUp { keycode: Some(Keycode::RShift), .. } => {
                self.is_zooming_cursor = false;
                self.input(cursor)
            }
            Event::MouseMotion { x, y, .. } if self.is_mousing_cursor => {
                Command::SetCursorTo(cursor_from_mouse(self.viewport, x, y))
            }
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                Command::SetCursorTo(Point {
                    x: cursor.x,
                    y: cursor.y.saturating_sub(if self.is_zooming_cursor { 10 } else { 1 }),
                })
            }
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                Command::SetCursorTo(Point {
                    x: cursor.x,
                    y: min(HEIGHT,
                           cursor.y.saturating_add(if self.is_zooming_cursor { 10 } else { 1 })),
                })
            }
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                Command::SetCursorTo(Point {
                    x: cursor.x.saturating_sub(if self.is_zooming_cursor { 10 } else { 1 }),
                    y: cursor.y,
                })
            }
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                Command::SetCursorTo(Point {
                    x: min(WIDTH,
                           cursor.x.saturating_add(if self.is_zooming_cursor { 10 } else { 1 })),
                    y: cursor.y,
                })
            }

            // Debug
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

            // Ignore unexpected input.
            _ => self.input(cursor),
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
            is_mousing_cursor: false,
            is_zooming_cursor: false,
        })
    }

    /// Render the cursor.
    fn render_cursor(&mut self, cursor: Point) {
        let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();
        let color = if self.is_mousing_cursor {
            Color::RGB(150, 150, 255)
        } else {
            Color::RGB(255, 255, 255)
        };
        let surface = font.render("@").blended(color).unwrap();
        let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
        render_in(&mut self.renderer,
                  &mut texture,
                  cell_rect(self.viewport, cursor),
                  true,
                  true);
    }

    /// Render the message log.
    fn render_log(&mut self, world: &World) {
        let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();

        let color = Color::RGB(255, 255, 255);
        let mut done = 0;
        for msg in world.messages.iter().take(LOG_ENTRIES_VISIBLE) {
            let bbox = Rect::new(0,
                                 ((LOG_ENTRIES_VISIBLE - done - 1) * CELL_PIXEL_HEIGHT) as i32,
                                 SCREEN_WIDTH,
                                 CELL_PIXEL_HEIGHT as u32);
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
                            self.viewport,
                            here,
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
               viewport: Point,
               cell: Point,
               s: Option<Static>,
               m: Option<&Mobile>,
               color: Color) {
    let rect = cell_rect(viewport, cell);
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
        Static::GStoreCounter => ('', Color::RGB(133, 94, 66), None),
        Static::InnCounter => ('', Color::RGB(133, 94, 66), None),
        Static::Dungeon => ('ห', Color::RGB(129, 26, 26), Some(Color::RGB(66, 66, 111))),
        Static::Bed => ('Θ', Color::RGB(166, 128, 100), None),
        Static::Wall => ('#', Color::RGB(0, 0, 0), Some(Color::RGB(133, 94, 66))),
        Static::Door => ('║', Color::RGB(0, 0, 0), Some(Color::RGB(133, 94, 66))),
    };
    render_occupant(renderer, font, rect, ch, foreground, background)
}

/// Render a mobile.
fn render_mobile(_: &mut Renderer<'static>, _: &Font, _: Rect, _: &Mobile) {
    // They don't exist yet!
}

/// Render an occupant of a cell.
fn render_occupant(renderer: &mut Renderer<'static>,
                   font: &Font,
                   rect: Rect,
                   ch: char,
                   foreground: Color,
                   background: Option<Color>) {
    if let Some(bg) = background {
        renderer.set_draw_color(bg);
        let _ = renderer.fill_rect(rect);
    }

    let surface = font.render(ch.to_string().as_str()).blended(foreground).unwrap();
    let mut texture = renderer.create_texture_from_surface(&surface).unwrap();
    render_in(renderer, &mut texture, rect, true, true);
}

/// Copy a texture into a bounding box, scaling it if necessary.
fn render_in(renderer: &mut Renderer<'static>,
             texture: &mut Texture,
             bbox: Rect,
             center_horiz: bool,
             center_vert: bool) {
    let TextureQuery { mut width, mut height, .. } = texture.query();

    // Scale down.
    let wr = width as f32 / bbox.width() as f32;
    let hr = height as f32 / bbox.height() as f32;
    if width > bbox.width() || height > bbox.height() {
        if wr > hr {
            width = bbox.width();
            height = (height as f32 / wr).round() as u32;
        } else {
            width = (width as f32 / hr).round() as u32;
            height = bbox.height();
        }
    }

    // Center horizontally.
    let x = if center_horiz && width < bbox.width() {
        bbox.x() + (bbox.width() - width) as i32 / 2
    } else {
        bbox.x()
    };

    // Center vertically.
    let y = if center_vert && height < bbox.height() {
        bbox.y() + (bbox.height() - height) as i32 / 2
    } else {
        bbox.y()
    };

    // Render the texture in the target rect.
    let _ = renderer.copy(texture, None, Some(Rect::new(x, y, width, height)));
}

/// Get the `Rect` for a cell.
fn cell_rect(viewport: Point, cell: Point) -> Rect {
    let x = (cell.x - viewport.x) * CELL_PIXEL_WIDTH;
    let y = (LOG_ENTRIES_VISIBLE + LOG_GAP + cell.y - viewport.y) * CELL_PIXEL_HEIGHT;
    Rect::new(x as i32,
              y as i32,
              CELL_PIXEL_WIDTH as u32,
              CELL_PIXEL_HEIGHT as u32)
}

/// Get the cursor position from the mouse.
fn cursor_from_mouse(viewport: Point, x: i32, y: i32) -> Point {
    let cell_x = x as usize / CELL_PIXEL_WIDTH;
    let cell_y = (y as usize / CELL_PIXEL_HEIGHT).saturating_sub(LOG_ENTRIES_VISIBLE + LOG_GAP);
    Point {
        x: cell_x + viewport.x,
        y: cell_y + viewport.y,
    }
}

//! A renderer using SDL2.

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::*;
use sdl2;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use statics::*;
use std::cmp;
use std::collections::BTreeMap;
use std::f64;
use std::path::Path;
use templates::*;
use types::*;
use ui::UI;

// The font
const FONT_PATH: &'static str = "FSEX300.ttf";
const FONT_SIZE: u16 = 12;

// Size of the visible viewport, in grid cells.
const DEFAULT_VIEWPORT_CELL_HEIGHT: u32 = 50;
const DEFAULT_VIEWPORT_CELL_WIDTH: u32 = 100;

// Everything is done in terms of rows and columns, which are made of
// fixed-size cells.
const DEFAULT_CELL_PIXEL_HEIGHT: u32 = 16;
const DEFAULT_CELL_PIXEL_WIDTH: u32 = 16;

// Thickness of a border, in cells.
const BORDER_THICKNESS: u32 = 1;

// Number of log entries to display above the map.
const LOG_ENTRIES_VISIBLE: u32 = 7;

// Number of cells to "overshoot" the cursor by when scrolling.
const SCROLL_OVERSHOOT: usize = 25;

/// A user interface using SDL2.
#[allow(missing_debug_implementations,missing_copy_implementations)]
pub struct SdlUI {
    /// The screen configuration.
    screen: Screen,
    /// The top-left of the viewport.
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
    is_mousing: bool,
    /// Whether we're zooming around or not (shift held down).
    is_zooming: bool,
    /// Whether we're scrolling the viewport or not (alt held down).
    is_scrolling: bool,
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
            x: DEFAULT_VIEWPORT_CELL_WIDTH as usize / 2,
            y: DEFAULT_VIEWPORT_CELL_HEIGHT as usize / 2,
        }
    }

    fn render(&mut self, mobs: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();

        // Scroll the viewport if the cursor is outside of it.
        self.scroll_viewport(world.cursor, SCROLL_OVERSHOOT);

        // Render the message log.
        self.render_log(world);

        // Render the world OR heatmap.
        self.render_world(mobs, maps, world);

        // Render the selected template.
        if let Some(ref tpl) = world.template {
            self.render_template(world.cursor, &tpl)
        }

        // Display the cursor on top of everything else.
        self.render_cursor(world.cursor);

        // The window frame
        render_border(&mut self.renderer,
                      self.screen,
                      Rect::new(0, 0, self.screen.pixel_width(), self.screen.pixel_height()));

        // Finally, display everything.
        self.renderer.present();
    }

    fn input(&mut self, cursor: Point) -> Command {
        // Because typing out the full form of everything gets tedious.
        macro_rules! keydown {
            ( $k:ident ) => ( Event::KeyDown{keycode:Some(Keycode::$k), ..} )
        }
        macro_rules! keyup {
            ( $k:ident ) => ( Event::KeyUp{keycode:Some(Keycode::$k), ..} )
        }
        macro_rules! flag_set {
            ( $var:ident ) => ( { self.$var = true; Command::Render } )
        }
        macro_rules! flag_unset {
            ( $var:ident ) => ( { self.$var = false; Command::Render } )
        }

        let step = if self.is_zooming { 10 } else { 1 };

        match self.events.wait_event() {
            // Flags (setting/unsetting causes a rerender)
            keydown!(LShift) | keydown!(RShift) => flag_set!(is_zooming),
            keyup!(LShift) | keyup!(RShift) => flag_unset!(is_zooming),
            keydown!(LAlt) | keydown!(RAlt) => flag_set!(is_scrolling),
            keyup!(LAlt) | keyup!(RAlt) => flag_unset!(is_scrolling),

            // Window
            Event::Window { win_event: WindowEvent::Resized(w, h), .. } |
            Event::Window { win_event: WindowEvent::SizeChanged(w, h), .. } => {
                let cell_width = w as u32 / self.screen.cell_pixel_width;
                let cell_height = h as u32 / self.screen.cell_pixel_height;

                // Shrink the rendering area.
                self.screen.viewport_width = cell_width.saturating_sub(2 * BORDER_THICKNESS);
                self.screen.viewport_height =
                    cell_height.saturating_sub(3 * BORDER_THICKNESS + LOG_ENTRIES_VISIBLE);

                Command::Render
            }

            // Building
            keydown!(Return) => Command::BuildTemplate,

            // Cursor and Viewport
            Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                self.is_mousing = !self.is_mousing;
                if self.is_mousing {
                    Command::SetCursorTo(cursor_from_mouse(self.screen, x, y))
                } else {
                    Command::Render
                }
            }
            Event::MouseMotion { x, y, .. } if self.is_mousing => {
                Command::SetCursorTo(cursor_from_mouse(self.screen, x, y))
            }
            keydown!(Up) => {
                if self.is_scrolling {
                    self.screen.viewport_top_left.y =
                        self.screen.viewport_top_left.y.saturating_sub(step);
                    self.scroll_viewport(cursor, 0);
                    Command::Render
                } else {
                    Command::SetCursorTo(Point {
                        x: cursor.x,
                        y: cursor.y.saturating_sub(step),
                    })
                }
            }
            keydown!(Down) => {
                if self.is_scrolling {
                    self.screen.viewport_top_left.y =
                        cmp::min(HEIGHT, self.screen.viewport_top_left.y.saturating_add(step));
                    self.scroll_viewport(cursor, 0);
                    Command::Render
                } else {
                    Command::SetCursorTo(Point {
                        x: cursor.x,
                        y: cmp::min(HEIGHT, cursor.y.saturating_add(step)),
                    })
                }
            }
            keydown!(Left) => {
                if self.is_scrolling {
                    self.screen.viewport_top_left.x =
                        self.screen.viewport_top_left.x.saturating_sub(step);
                    self.scroll_viewport(cursor, 0);
                    Command::Render
                } else {
                    Command::SetCursorTo(Point {
                        x: cursor.x.saturating_sub(step),
                        y: cursor.y,
                    })
                }
            }
            keydown!(Right) => {
                if self.is_scrolling {
                    self.screen.viewport_top_left.x =
                        cmp::min(WIDTH, self.screen.viewport_top_left.x.saturating_add(step));
                    self.scroll_viewport(cursor, 0);
                    Command::Render
                } else {
                    Command::SetCursorTo(Point {
                        x: cmp::min(WIDTH, cursor.x.saturating_add(step)),
                        y: cursor.y,
                    })
                }
            }

            // Debug
            keydown!(F5) => Command::Render,
            keydown!(Tab) => {
                self.active_heatmap = next_heatmap(self.active_heatmap);
                println!("DEBUG: rendering heatmap {:?}", self.active_heatmap);
                Command::Render
            }
            keydown!(Semicolon) => {
                self.show_heatmap = !self.show_heatmap;
                println!("DEBUG: toggling heatmap to {:?}", self.show_heatmap);
                Command::Render
            }

            // Exit
            Event::Quit { .. } |
            Event::AppTerminating { .. } |
            keydown!(Escape) => Command::Quit,
            keydown!(Space) => Command::Skip,

            // Ignore unexpected input.
            _ => self.input(cursor),
        }
    }
}

impl SdlUI {
    /// Construct a new SDL2 interface. Should only be called once.
    pub fn new() -> Result<SdlUI, String> {
        // SDL stuff doesn't implement the trait `try!` needs.
        macro_rules! ftry {
            ( $e:expr ) => ( match $e {
                Ok(x) => x,
                Err(e) => return Err(format!("{}", e)),
            } )
        }

        let screen = Screen {
            cell_pixel_width: DEFAULT_CELL_PIXEL_WIDTH,
            cell_pixel_height: DEFAULT_CELL_PIXEL_HEIGHT,
            viewport_width: DEFAULT_VIEWPORT_CELL_WIDTH,
            viewport_height: DEFAULT_VIEWPORT_CELL_HEIGHT,
            viewport_top_left: Point { x: 0, y: 0 },
        };

        let context = try!(sdl2::init());
        let events = try!(context.event_pump());
        let video = try!(context.video());
        let window = ftry!(video.window("Rogue Mayor", screen.pixel_width(), screen.pixel_height())
            .position_centered()
            .opengl()
            .resizable()
            .build());
        let renderer = ftry!(window.renderer().build());
        let ttf = ftry!(sdl2::ttf::init());

        Ok(SdlUI {
            screen: screen,
            context: context,
            events: events,
            video: video,
            renderer: renderer,
            ttf: ttf,
            show_heatmap: false,
            active_heatmap: (Style::Approach, MapTag::Adventure),
            is_mousing: false,
            is_zooming: false,
            is_scrolling: false,
        })
    }

    /// Render the cursor.
    fn render_cursor(&mut self, cursor: Point) {
        if let Some(cursor_pos) = self.screen.to_screenpos(cursor) {
            let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();
            let color = if self.is_mousing {
                Color::RGB(150, 150, 255)
            } else {
                Color::RGB(255, 255, 255)
            };
            let surface = font.render("@").blended(color).unwrap();
            let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
            render_in(&mut self.renderer,
                      &mut texture,
                      cursor_pos.rect(),
                      true,
                      true);
        }
    }

    /// Render the message log.
    fn render_log(&mut self, world: &World) {
        let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();

        let color = Color::RGB(255, 255, 255);
        let mut done = 0;
        for msg in world.messages.iter().take(LOG_ENTRIES_VISIBLE as usize) {
            let bbox = Rect::new((BORDER_THICKNESS * self.screen.cell_pixel_width) as i32,
                                 ((LOG_ENTRIES_VISIBLE - done - 1 + BORDER_THICKNESS) *
                                  self.screen
                                     .cell_pixel_height) as i32,
                                 self.screen.pixel_width(),
                                 self.screen.cell_pixel_height);
            let surface = font.render(msg.msg.as_str()).blended(color).unwrap();
            let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
            render_in(&mut self.renderer, &mut texture, bbox, false, true);
            done += 1;
        }

        render_border(&mut self.renderer,
                      self.screen,
                      Rect::new(0,
                                0,
                                self.screen.pixel_width(),
                                (LOG_ENTRIES_VISIBLE + 2) * self.screen.cell_pixel_height));
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
        let min_y = self.screen.viewport_top_left.y;
        let min_x = self.screen.viewport_top_left.x;
        for y in min_y..cmp::min(HEIGHT, min_y + self.screen.viewport_height as usize) {
            for x in min_x..cmp::min(WIDTH, min_x + self.screen.viewport_width as usize) {
                let here = Point { x: x, y: y };
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
                            self.screen,
                            here,
                            world.statics.at(here),
                            mobs.get(&here),
                            Some(color));
            }
        }
    }

    /// Render a template at the cursor.
    fn render_template(&mut self, cursor: Point, tpl: &Template) {
        let font = self.ttf.load_font(Path::new(FONT_PATH), FONT_SIZE).unwrap();
        for (p, &(s, _)) in &tpl.components {
            render_cell(&mut self.renderer,
                        &font,
                        self.screen,
                        p.offset(cursor),
                        Some(s),
                        None,
                        None);
        }
    }

    /// Scroll the viewport, with the given amount of overshoot, to fit the cursor.
    fn scroll_viewport(&mut self, cursor: Point, overshoot: usize) {
        if cursor.x < self.screen.viewport_top_left.x {
            self.screen.viewport_top_left.x = cursor.x.saturating_sub(overshoot);
        } else if cursor.x >=
                  self.screen.viewport_top_left.x + self.screen.viewport_width as usize {
            self.screen.viewport_top_left.x = cmp::min(WIDTH - self.screen.viewport_width as usize,
                                                       cursor.x + overshoot -
                                                       self.screen.viewport_width as usize)
        }
        if cursor.y < self.screen.viewport_top_left.y {
            self.screen.viewport_top_left.y = cursor.y.saturating_sub(overshoot);
        } else if cursor.y >=
                  self.screen.viewport_top_left.y + self.screen.viewport_height as usize {
            self.screen.viewport_top_left.y =
                cmp::min(HEIGHT - self.screen.viewport_height as usize,
                         cursor.y + overshoot - self.screen.viewport_height as usize)
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

/// Render a border. This does not render over the area within the border.
fn render_border(renderer: &mut Renderer<'static>, screen: Screen, bbox: Rect) {
    let hthickness = BORDER_THICKNESS * screen.cell_pixel_width;
    let vthickness = BORDER_THICKNESS * screen.cell_pixel_height;

    let x1 = bbox.x();
    let y1 = bbox.y();
    let x2 = x1 + bbox.width() as i32 - hthickness as i32;
    let y2 = y1 + bbox.height() as i32 - vthickness as i32;

    renderer.set_draw_color(Color::RGB(115, 115, 115));

    let _ = renderer.fill_rect(Rect::new(x1, y1, bbox.width(), vthickness));
    let _ = renderer.fill_rect(Rect::new(x1, y1, hthickness, bbox.height()));
    let _ = renderer.fill_rect(Rect::new(x1, y2, bbox.width(), vthickness));
    let _ = renderer.fill_rect(Rect::new(x2, y1, hthickness, bbox.height()));
}

/// Render a cell.
fn render_cell(renderer: &mut Renderer<'static>,
               font: &Font,
               screen: Screen,
               cell: Point,
               s: Option<Static>,
               m: Option<&Mobile>,
               color: Option<Color>) {
    if let Some(cell_screenpos) = screen.to_screenpos(cell) {
        let rect = cell_screenpos.rect();

        if let Some(c) = color {
            renderer.set_draw_color(c);
            let _ = renderer.fill_rect(rect);
        }

        match (m, s) {
            (Some(mob), _) => render_mobile(renderer, font, rect, mob),
            (_, Some(stat)) => render_static(renderer, font, rect, &stat),
            _ => {}
        }
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

/// Get the cursor position from the mouse.
fn cursor_from_mouse(screen: Screen, x: i32, y: i32) -> Point {
    let cell_x = (x as u32 / screen.cell_pixel_width).saturating_sub(BORDER_THICKNESS);
    let cell_y = (y as u32 / screen.cell_pixel_height)
        .saturating_sub(LOG_ENTRIES_VISIBLE + BORDER_THICKNESS * 2);
    Point {
        x: cell_x as usize + screen.viewport_top_left.x,
        y: cell_y as usize + screen.viewport_top_left.y,
    }
}

/// The screen. Width and height are measured in cells.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Screen {
    cell_pixel_width: u32,
    cell_pixel_height: u32,
    viewport_width: u32,
    viewport_height: u32,
    viewport_top_left: Point,
}

impl Screen {
    /// Width of the screen in pixels.
    fn pixel_width(&self) -> u32 {
        (self.viewport_width + BORDER_THICKNESS * 2) * self.cell_pixel_width
    }

    /// Height of the screen in pixels.
    fn pixel_height(&self) -> u32 {
        (self.viewport_height + LOG_ENTRIES_VISIBLE + BORDER_THICKNESS * 3) * self.cell_pixel_height
    }

    /// Turn a point in the world into a `ScreenPos`, if it's on screen.
    fn to_screenpos(&self, p: Point) -> Option<ScreenPos> {
        if p.x < self.viewport_top_left.x || p.y < self.viewport_top_left.y ||
           p.x >= self.viewport_top_left.x + self.viewport_width as usize ||
           p.y >= self.viewport_top_left.y + self.viewport_height as usize {
            None
        } else {
            Some(ScreenPos {
                x: (p.x - self.viewport_top_left.x) as u32,
                y: (p.y - self.viewport_top_left.y) as u32,
                cell_pixel_width: self.cell_pixel_width,
                cell_pixel_height: self.cell_pixel_height,
            })
        }
    }
}

/// Screen positions, as CELL_PIXEL_WIDTHxCELL_PIXEL_HEIGHT boxes. This is only valid until the
/// screen is next resized.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ScreenPos {
    x: u32,
    y: u32,
    // Taken from the `Screen` at creation time.
    cell_pixel_width: u32,
    cell_pixel_height: u32,
}

impl ScreenPos {
    /// Get the `Rect` that this position corresponds to.
    fn rect(&self) -> Rect {
        Rect::new(((self.x + BORDER_THICKNESS) * self.cell_pixel_width) as i32,
                  ((self.y + LOG_ENTRIES_VISIBLE + BORDER_THICKNESS * 2) *
                   self.cell_pixel_height) as i32,
                  self.cell_pixel_width,
                  self.cell_pixel_height)
    }
}

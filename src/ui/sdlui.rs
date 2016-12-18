//! A renderer using SDL2.

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::*;
use sdl2;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use sdl2::event::{Event, WindowEvent};
use sdl2::gfx::framerate::FPSManager;
use sdl2::image::{INIT_PNG, LoadTexture, Sdl2ImageContext};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Renderer, Texture, TextureQuery};
use statics::*;
use std::cmp;
use std::collections::BTreeMap;
use std::f64;
use std::path::Path;
use templates::*;
use types::*;
use ui::UI;

// The font
const FONT_PATH: &'static str = "font.png";
const FONT_CHAR_WIDTH: u8 = 16;
const FONT_PIXEL_WIDTH: u32 = 12;
const FONT_PIXEL_HEIGHT: u32 = 12;
const FONT_PIXEL_OFF_HORIZ: u32 = 2;
const FONT_PIXEL_OFF_VERT: u32 = 0;

// Size of the visible viewport, in cells.
const DEFAULT_VIEWPORT_CELL_HEIGHT: u32 = 50;
const DEFAULT_VIEWPORT_CELL_WIDTH: u32 = 100;

// Everything is done in terms of rows and columns, which are made of
// fixed-size cells.
const DEFAULT_CELL_PIXEL_HEIGHT: u32 = FONT_PIXEL_HEIGHT - 2 * FONT_PIXEL_OFF_VERT;
const DEFAULT_CELL_PIXEL_WIDTH: u32 = FONT_PIXEL_WIDTH - 2 * FONT_PIXEL_OFF_HORIZ;

// The framerate
const DEFAULT_FRAMERATE: u32 = 15;

// Width of the sidebar, in cells.
const SIDEBAR_WIDTH: u32 = 25;

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
    /// The FPS manager.
    fps: FPSManager,
    /// The image context.
    image: Sdl2ImageContext,
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
    /// What to display in the sidebar.
    menu: Menu,
    /// Increments (wrapping) on every frame.
    indicator: u8,
}

/// What menu to display in the sidebar.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Menu {
    Main,
    Template,
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
        self.screen.clear();

        // Scroll the viewport if the cursor is outside of it.
        self.scroll_viewport(world.cursor, SCROLL_OVERSHOOT);

        // Render the message log.
        self.render_log(world);

        // Render the help/control sidebar.
        self.render_sidebar();

        // Render the world OR heatmap.
        self.render_world(mobs, maps, world);

        // Render the selected template.
        if let Some(ref tpl) = world.template {
            self.render_template(world.cursor, &tpl)
        }

        // Display the cursor on top of everything else.
        self.render_cursor(world.cursor);

        // The window frame
        let full_width = self.screen.cell_width();
        let full_height = self.screen.cell_height();
        self.screen.render_border(ScreenRect::new(0, 0, full_width, full_height));

        // The indicator.
        let texture = self.screen.render_bytes(&[self.indicator], Color::RGB(150, 200, 250));
        self.screen.render_in_cell(&texture, ScreenPos { x: 0, y: 0 });
        self.indicator = self.indicator.wrapping_add(1);

        // Finally, display everything.
        self.screen.present();
    }

    fn input(&mut self, cursor: Point) -> Command {
        // Wait until it's time for the next frame.
        let _ = self.fps.delay();

        // Check for an event.
        if let Some(event) = self.events.poll_event() {
            self.input_handler(event, cursor)
        } else {
            Command::Skip
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

        let mut screen = Screen {
            renderer: None,
            font: None,
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
        let image = ftry!(sdl2::image::init(INIT_PNG));

        // Set the framerate.
        let mut fps = FPSManager::new();
        try!(fps.set_framerate(DEFAULT_FRAMERATE));

        // Finish creating the screen:
        let renderer = ftry!(window.renderer().target_texture().build());
        let font = try!(renderer.load_texture(Path::new(FONT_PATH)));
        screen.renderer = Some(renderer);
        screen.font = Some(font);

        Ok(SdlUI {
            screen: screen,
            context: context,
            events: events,
            video: video,
            fps: fps,
            image: image,
            show_heatmap: false,
            active_heatmap: (Style::Approach, MapTag::Adventure),
            is_mousing: false,
            is_zooming: false,
            is_scrolling: false,
            menu: Menu::Main,
            indicator: 0,
        })
    }

    /// Handle input
    fn input_handler(&mut self, event: Event, cursor: Point) -> Command {
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

        match event {
            // Flags (setting/unsetting causes a rerender)
            keydown!(LShift) | keydown!(RShift) => flag_set!(is_zooming),
            keyup!(LShift) | keyup!(RShift) => flag_unset!(is_zooming),
            keydown!(LAlt) | keydown!(RAlt) => flag_set!(is_scrolling),
            keyup!(LAlt) | keyup!(RAlt) => flag_unset!(is_scrolling),

            // Menu
            keydown!(B) => {
                if self.menu == Menu::Main {
                    self.menu = Menu::Template
                }
                Command::Render
            }
            keydown!(G) => {
                if self.menu == Menu::Template {
                    Command::SetTemplateTo(Templates::GeneralStore)
                } else {
                    Command::Render
                }
            }
            keydown!(I) => {
                if self.menu == Menu::Template {
                    Command::SetTemplateTo(Templates::Inn)
                } else {
                    Command::Render
                }
            }
            keydown!(Escape) => {
                self.menu = Menu::Main;
                Command::Render
            }
            keydown!(Return) => {
                if self.menu == Menu::Template {
                    Command::BuildTemplate
                } else {
                    Command::Render
                }
            }

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

            // Cursor and Viewport
            Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                self.is_mousing = !self.is_mousing;
                if self.is_mousing {
                    Command::SetCursorTo(self.screen.cursor_from_mouse(x, y))
                } else {
                    Command::Render
                }
            }
            Event::MouseMotion { x, y, .. } if self.is_mousing => {
                Command::SetCursorTo(self.screen.cursor_from_mouse(x, y))
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
            Event::AppTerminating { .. } => Command::Quit,
            keydown!(Space) => Command::Skip,

            // Ignore unexpected input.
            _ => self.input(cursor),
        }
    }

    /// Render the cursor.
    fn render_cursor(&mut self, cursor: Point) {
        if let Some(cursor_pos) = self.screen.to_screenpos(cursor) {
            let color = if self.is_mousing {
                Color::RGB(150, 150, 255)
            } else {
                Color::RGB(255, 255, 255)
            };
            let texture = self.screen.render_text('@'.to_string(), color);
            self.screen.render_in_cell(&texture, cursor_pos);
        }
    }

    /// Render the message log.
    fn render_log(&mut self, world: &World) {
        let log_width = self.screen.cell_width() - BORDER_THICKNESS - SIDEBAR_WIDTH;
        let log_height = LOG_ENTRIES_VISIBLE + 2 * BORDER_THICKNESS;

        let color = Color::RGB(255, 255, 255);
        let mut done = 0;
        for msg in world.messages.iter().take(LOG_ENTRIES_VISIBLE as usize) {
            let bbox = ScreenRect::new(BORDER_THICKNESS,
                                       ((LOG_ENTRIES_VISIBLE - done - 1 + BORDER_THICKNESS)),
                                       log_width - 2,
                                       1);
            let texture = self.screen.render_text(msg.msg.clone(), color);
            self.screen.render_in_rect(&texture, bbox, false, true);
            done += 1;
        }

        self.screen.render_border(ScreenRect::new(0, 0, log_width, log_height));
    }

    /// Render the sidebar.
    fn render_sidebar(&mut self) {
        let sidebar_x = self.screen.cell_width() - 2 * BORDER_THICKNESS - SIDEBAR_WIDTH;
        let sidebar_width = SIDEBAR_WIDTH + 2 * BORDER_THICKNESS;
        let sidebar_height = self.screen.cell_height();

        let controls = match self.menu {
            Menu::Main => vec![vec![("b", "Building")]],
            Menu::Template => {
                vec![vec![("g", "General Store"), ("i", "Inn")],
                     vec![("RET", "Build at cursor"), ("ESC", "Return to main menu")]]
            }
        };

        let mut y = 2;
        for cs in controls {
            for (key, text) in cs {
                let mut pos = ScreenPos {
                    x: sidebar_x + 2,
                    y: y,
                };

                // Key
                let mut texture = self.screen
                    .render_text(key.to_string(), Color::RGB(100, 255, 100));
                self.screen.render_in_rect(&texture,
                                           ScreenRect::new(pos.x, pos.y, key.len() as u32, 1),
                                           false,
                                           true);

                // Colon
                pos.x += key.len() as u32;
                texture = self.screen.render_text(':'.to_string(), Color::RGB(255, 255, 255));
                self.screen.render_in_cell(&texture, pos);

                // Text
                pos.x += 2;
                texture = self.screen.render_text(text.to_string(), Color::RGB(255, 255, 255));
                self.screen.render_in_rect(&texture,
                                           ScreenRect::new(pos.x, pos.y, sidebar_width, 1),
                                           false,
                                           true);
                y += 1;
            }
            y += 1;
        }

        self.screen.render_border(ScreenRect::new(sidebar_x, 0, sidebar_width, sidebar_height));
    }

    /// Render the world, with a heatmap overlay if enabled.
    fn render_world(&mut self, mobs: &BTreeMap<Point, Mobile>, maps: &Maps, world: &World) {
        // The world coordinates that fit on screen.
        let min_y = self.screen.viewport_top_left.y;
        let min_x = self.screen.viewport_top_left.x;
        let max_y = cmp::min(HEIGHT, min_y + self.screen.viewport_height as usize);
        let max_x = cmp::min(WIDTH, min_x + self.screen.viewport_width as usize);

        // Render the active heatmap.
        let (style, tag) = self.active_heatmap;
        let heatmap = maps.get(tag);
        let map = match style {
            Style::Approach => &heatmap.approach,
            Style::FleeCowardly => &heatmap.flee_cowardly,
            Style::FleeBravely => &heatmap.flee_bravely,
        };

        // Find the min and max values in the visible heatmap.
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        if self.show_heatmap {
            for y in min_y..max_y {
                for x in min_x..max_x {
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
        for y in min_y..max_y {
            for x in min_x..max_x {
                let here = Point { x: x, y: y };
                if let Some(screenpos) = self.screen.to_screenpos(here) {
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
                    self.render_cell(screenpos,
                                     world.statics.at(here),
                                     mobs.get(&here),
                                     Some(color));
                }
            }
        }
    }

    /// Render a template at the cursor.
    fn render_template(&mut self, cursor: Point, tpl: &Template) {
        for (p, &(s, _)) in &tpl.components {
            if let Some(screenpos) = self.screen.to_screenpos(p.offset(cursor)) {
                self.render_cell(screenpos, Some(s), None, None);
            }
        }
    }

    /// Render a cell.
    fn render_cell(&mut self,
                   screenpos: ScreenPos,
                   s: Option<Static>,
                   m: Option<&Mobile>,
                   color: Option<Color>) {
        let mut background = color;
        let mut surface = None;

        let to_render = match (m, s) {
            (Some(mob), _) => Some(mob.visual()),
            (_, Some(stat)) => Some(stat.visual()),
            _ => None,
        };

        if let Some((b, fgcol, bgcol)) = to_render {
            if bgcol.is_some() {
                background = bgcol
            }

            surface = Some(self.screen.render_bytes(&[b], fgcol));
        }

        // Render the background color, if there is one.
        if let Some(bg) = background {
            self.screen.fill_rect(screenpos.rect(), bg);
        }

        // Render the surface, if there is one.
        if let Some(ref sface) = surface {
            self.screen.render_in_cell(sface, screenpos);
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


// ********** Screens **********

/// The screen. Width and height are measured in cells.
#[allow(missing_copy_implementations, missing_debug_implementations)]
struct Screen {
    // It is an error for either of these to be `None`, they are set immediately after creating the
    // renderer. The issue is that the `Screen` needs to be creatable, so `Screen::pixel_width()`
    // and `Screen::pixel_height()` can be called to determine the dimensions of the initial window,
    // but the renderer (and therefore font) cannot be created before the window. So the `Screen`
    // must first be created with a dummy renderer (and font).
    renderer: Option<Renderer<'static>>,
    font: Option<Texture>,

    cell_pixel_width: u32,
    cell_pixel_height: u32,
    viewport_width: u32,
    viewport_height: u32,
    viewport_top_left: Point,
}

impl Screen {
    // ******************** GEOMETRY ********************

    /// Width of the screen in cells.
    fn cell_width(&self) -> u32 {
        self.viewport_width + SIDEBAR_WIDTH + BORDER_THICKNESS * 3
    }

    /// Height of the screen in cells.
    fn cell_height(&self) -> u32 {
        self.viewport_height + LOG_ENTRIES_VISIBLE + BORDER_THICKNESS * 3
    }

    /// Width of the screen in pixels.
    fn pixel_width(&self) -> u32 {
        self.cell_width() * self.cell_pixel_width
    }

    /// Height of the screen in pixels.
    fn pixel_height(&self) -> u32 {
        self.cell_height() * self.cell_pixel_height
    }

    /// Turn a point in the world into a `ScreenPos`, if it's on screen.
    fn to_screenpos(&self, p: Point) -> Option<ScreenPos> {
        if p.x < self.viewport_top_left.x || p.y < self.viewport_top_left.y ||
           p.x >= self.viewport_top_left.x + self.viewport_width as usize ||
           p.y >= self.viewport_top_left.y + self.viewport_height as usize {
            None
        } else {
            Some(ScreenPos {
                x: (p.x - self.viewport_top_left.x) as u32 + BORDER_THICKNESS,
                y: (p.y - self.viewport_top_left.y) as u32 + 2 * BORDER_THICKNESS +
                   LOG_ENTRIES_VISIBLE,
            })
        }
    }

    /// Get the cursor position from the mouse.
    fn cursor_from_mouse(&self, x: i32, y: i32) -> Point {
        let cell_x = (x as u32 / self.cell_pixel_width).saturating_sub(BORDER_THICKNESS);
        let cell_y = (y as u32 / self.cell_pixel_height)
            .saturating_sub(LOG_ENTRIES_VISIBLE + BORDER_THICKNESS * 2);
        Point {
            x: cell_x as usize + self.viewport_top_left.x,
            y: cell_y as usize + self.viewport_top_left.y,
        }
    }

    // ******************** RENDERING ********************

    /// Clear the screen.
    fn clear(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_draw_color(Color::RGB(0, 0, 0));
            renderer.clear();
        }
    }

    /// Display the rendered screen.
    fn present(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.present();
        }
    }

    /// Render a border. This does not render over the area within the border.
    fn render_border(&mut self, bbox: ScreenRect) {
        let border_color = Color::RGB(115, 115, 115);

        let top = ScreenRect::new(bbox.top_left.x,
                                  bbox.top_left.y,
                                  bbox.width,
                                  BORDER_THICKNESS);
        let bottom = ScreenRect::new(bbox.top_left.x,
                                     bbox.top_left.y + bbox.height - BORDER_THICKNESS,
                                     bbox.width,
                                     BORDER_THICKNESS);
        let left = ScreenRect::new(bbox.top_left.x,
                                   bbox.top_left.y,
                                   BORDER_THICKNESS,
                                   bbox.height);
        let right = ScreenRect::new(bbox.top_left.x + bbox.width - BORDER_THICKNESS,
                                    bbox.top_left.y,
                                    BORDER_THICKNESS,
                                    bbox.height);

        self.fill_rect(top, border_color);
        self.fill_rect(bottom, border_color);
        self.fill_rect(left, border_color);
        self.fill_rect(right, border_color);
    }

    /// Fill a rect with a colour.
    fn fill_rect(&mut self, srect: ScreenRect, color: Color) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_draw_color(color);
            let _ = renderer.fill_rect(srect.rect(self.cell_pixel_width, self.cell_pixel_height));
        }
    }

    fn render_text(&mut self, text: String, color: Color) -> Texture {
        self.render_bytes(text.as_bytes(), color)
    }

    fn render_bytes(&mut self, bytes: &[u8], color: Color) -> Texture {
        if let Some(ref mut renderer) = self.renderer {
            let mut texture = renderer.create_texture_target(PixelFormatEnum::ARGB8888,
                                       self.cell_pixel_width * bytes.len() as u32,
                                       self.cell_pixel_height)
                .unwrap();

            // Make the texture transparent.
            texture.set_blend_mode(BlendMode::Blend);

            // Set the color mod
            let (r, g, b) = color.rgb();
            texture.set_color_mod(r, g, b);

            let _ = renderer.render_target()
                .unwrap()
                .set(texture);

            // Clear the texture
            renderer.set_draw_color(Color::RGBA(0, 0, 0, 0));
            renderer.clear();

            if let Some(ref font) = self.font {
                let mut dst = Rect::new(0, 0, self.cell_pixel_width, self.cell_pixel_height);
                for b in bytes {
                    let x = (b % FONT_CHAR_WIDTH) as u32;
                    let y = (b / FONT_CHAR_WIDTH) as u32;
                    let src = Rect::new((x * FONT_PIXEL_WIDTH + FONT_PIXEL_OFF_HORIZ) as i32,
                                        (y * FONT_PIXEL_HEIGHT + FONT_PIXEL_OFF_VERT) as i32,
                                        FONT_PIXEL_WIDTH - 2 * FONT_PIXEL_OFF_HORIZ,
                                        FONT_PIXEL_HEIGHT - 2 * FONT_PIXEL_OFF_VERT);
                    let _ = renderer.copy(&font, Some(src), Some(dst));
                    let old_x = dst.x();
                    dst.set_x(old_x + self.cell_pixel_width as i32);
                }
            }

            renderer.render_target().unwrap().reset().unwrap().unwrap()
        } else {
            panic!("Renderer not instantiated!")
        }
    }

    // ******************** PRIMITIVE RENDERING ********************

    /// Render a texture in a cell.
    fn render_in_cell(&mut self, texture: &Texture, screenpos: ScreenPos) {
        self.render_in_rect(texture, screenpos.rect(), true, true)
    }

    /// Copy a texture into a bounding box, scaling it if necessary.
    fn render_in_rect(&mut self,
                      texture: &Texture,
                      srect: ScreenRect,
                      center_horiz: bool,
                      center_vert: bool) {
        let bbox = srect.rect(self.cell_pixel_width, self.cell_pixel_height);

        if let Some(ref mut renderer) = self.renderer {
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
            let _ = renderer.copy(&texture, None, Some(Rect::new(x, y, width, height)));
        }
    }
}


// ********** Screen Coordinates **********

/// Screen positions, as CELL_PIXEL_WIDTHxCELL_PIXEL_HEIGHT boxes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ScreenPos {
    x: u32,
    y: u32,
}

impl ScreenPos {
    /// Get the `ScreenRect` that this position corresponds to.
    fn rect(self) -> ScreenRect {
        ScreenRect {
            top_left: self,
            width: 1,
            height: 1,
        }
    }
}

/// Regions of the screen, measured in cells.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ScreenRect {
    top_left: ScreenPos,
    width: u32,
    height: u32,
}

impl ScreenRect {
    /// Construct a new `ScreenRect`.
    fn new(x: u32, y: u32, width: u32, height: u32) -> ScreenRect {
        ScreenRect {
            top_left: ScreenPos { x: x, y: y },
            width: width,
            height: height,
        }
    }

    /// Get the SDL `Rect` that this corresponds to.
    fn rect(&self, cell_pixel_width: u32, cell_pixel_height: u32) -> Rect {
        Rect::new((self.top_left.x * cell_pixel_width) as i32,
                  (self.top_left.y * cell_pixel_height) as i32,
                  self.width * cell_pixel_width,
                  self.height * cell_pixel_height)
    }
}

// ********** Renderable Things **********

trait Visual {
    /// What the thing should look like.
    fn visual(&self) -> (u8, Color, Option<Color>);
}

impl Visual for Mobile {
    fn visual(&self) -> (u8, Color, Option<Color>) {
        // There aren't any yet!
        ('m' as u8, Color::RGB(0, 255, 0), None)
    }
}

impl Visual for Static {
    fn visual(&self) -> (u8, Color, Option<Color>) {
        match *self {
            Static::GStoreCounter => (210, Color::RGB(133, 94, 66), None),
            Static::InnCounter => (210, Color::RGB(133, 94, 66), None),
            Static::Dungeon => (234, Color::RGB(129, 26, 26), Some(Color::RGB(66, 66, 111))),
            Static::Bed => (233, Color::RGB(166, 128, 100), None),
            Static::Wall => ('#' as u8, Color::RGB(0, 0, 0), Some(Color::RGB(133, 94, 66))),
            Static::Door => (186, Color::RGB(0, 0, 0), Some(Color::RGB(133, 94, 66))),
        }
    }
}


// ********** Utilities **********

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

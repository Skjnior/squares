//! Carrés contrôlés au clavier (projet PG bas niveau — Piston / Rust).
//!
//! Usage : `cargo run -- AZER QSDF WXCV`
//!
//! Chaque argument est un bloc de **4 lettres** = un carré.
//! Le sens des 4 lettres est fixé par `KEY_CONVENTION` (voir plus bas).

use std::collections::HashSet;
use std::process;

use glutin_window::GlutinWindow as Window;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, Key, PressEvent, ReleaseEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent,
};
use piston::window::WindowSettings;

const WINDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 600.0;
const SQUARE_SIZE: f64 = 48.0;
const MOVE_SPEED: f64 = 220.0;

/// Comment interpréter les 4 lettres d'un bloc (ex. `AZER`).
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum KeyConvention {
    /// 1ʳᵉ = haut, 2ᵉ = bas, 3ᵉ = gauche, 4ᵉ = droite.
    HautBasGaucheDroite,
    /// Comme les 4 touches d'une rangée AZERTY lues de gauche à droite :
    /// 1ʳᵉ = gauche, 2ᵉ = haut, 3ᵉ = bas, 4ᵉ = droite (`A Z E R` sur un clavier FR).
    RangeeAzerty,
}

/// **À modifier ici** si le prof précise un autre ordre pour les 4 lettres.
const KEY_CONVENTION: KeyConvention = KeyConvention::HautBasGaucheDroite;

fn controls_from_keys(keys: [Key; 4]) -> Controls {
    match KEY_CONVENTION {
        KeyConvention::HautBasGaucheDroite => Controls {
            up: keys[0],
            down: keys[1],
            left: keys[2],
            right: keys[3],
        },
        KeyConvention::RangeeAzerty => Controls {
            left: keys[0],
            up: keys[1],
            down: keys[2],
            right: keys[3],
        },
    }
}

/// Contrôles d'un carré : 4 touches (haut, bas, gauche, droite).
#[derive(Clone, Debug)]
struct Controls {
    up: Key,
    down: Key,
    left: Key,
    right: Key,
}

/// État d'un carré à l'écran.
struct Square {
    x: f64,
    y: f64,
    color: [f32; 4],
    controls: Controls,
    moving_up: bool,
    moving_down: bool,
    moving_left: bool,
    moving_right: bool,
}

impl Square {
    fn new(index: usize, total: usize, controls: Controls) -> Self {
        let cols = (total as f64).sqrt().ceil() as usize;
        let row = index / cols.max(1);
        let col = index % cols.max(1);
        let margin = 80.0;
        let span_x = WINDOW_WIDTH - 2.0 * margin - SQUARE_SIZE;
        let span_y = WINDOW_HEIGHT - 2.0 * margin - SQUARE_SIZE;
        let step_x = if cols <= 1 {
            0.0
        } else {
            span_x / (cols - 1) as f64
        };
        let rows = (total + cols - 1) / cols;
        let step_y = if rows <= 1 {
            0.0
        } else {
            span_y / (rows - 1) as f64
        };

        let palette: [[f32; 4]; 6] = [
            [0.95, 0.35, 0.35, 1.0],
            [0.35, 0.75, 0.45, 1.0],
            [0.35, 0.55, 0.95, 1.0],
            [0.95, 0.75, 0.25, 1.0],
            [0.75, 0.45, 0.85, 1.0],
            [0.25, 0.85, 0.85, 1.0],
        ];

        Self {
            x: margin + col as f64 * step_x,
            y: margin + row as f64 * step_y,
            color: palette[index % palette.len()],
            controls,
            moving_up: false,
            moving_down: false,
            moving_left: false,
            moving_right: false,
        }
    }

    fn apply_key(&mut self, key: Key, pressed: bool) {
        if key == self.controls.up {
            self.moving_up = pressed;
        } else if key == self.controls.down {
            self.moving_down = pressed;
        } else if key == self.controls.left {
            self.moving_left = pressed;
        } else if key == self.controls.right {
            self.moving_right = pressed;
        }
    }

    fn step(&mut self, dt: f64) {
        let mut dx = 0.0;
        let mut dy = 0.0;
        if self.moving_left {
            dx -= MOVE_SPEED * dt;
        }
        if self.moving_right {
            dx += MOVE_SPEED * dt;
        }
        if self.moving_up {
            dy -= MOVE_SPEED * dt;
        }
        if self.moving_down {
            dy += MOVE_SPEED * dt;
        }
        self.x += dx;
        self.y += dy;

        let max_x = WINDOW_WIDTH - SQUARE_SIZE;
        let max_y = WINDOW_HEIGHT - SQUARE_SIZE;
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }
}

fn key_from_char(c: char) -> Option<Key> {
    match c.to_ascii_uppercase() {
        'A' => Some(Key::A),
        'B' => Some(Key::B),
        'C' => Some(Key::C),
        'D' => Some(Key::D),
        'E' => Some(Key::E),
        'F' => Some(Key::F),
        'G' => Some(Key::G),
        'H' => Some(Key::H),
        'I' => Some(Key::I),
        'J' => Some(Key::J),
        'K' => Some(Key::K),
        'L' => Some(Key::L),
        'M' => Some(Key::M),
        'N' => Some(Key::N),
        'O' => Some(Key::O),
        'P' => Some(Key::P),
        'Q' => Some(Key::Q),
        'R' => Some(Key::R),
        'S' => Some(Key::S),
        'T' => Some(Key::T),
        'U' => Some(Key::U),
        'V' => Some(Key::V),
        'W' => Some(Key::W),
        'X' => Some(Key::X),
        'Y' => Some(Key::Y),
        'Z' => Some(Key::Z),
        _ => None,
    }
}

/// Analyse les arguments : blocs de 4 lettres, pas de touche en double.
fn parse_square_groups(args: &[String]) -> Result<Vec<Controls>, String> {
    if args.is_empty() {
        return Err(
            "usage : squares <4 lettres> [<4 lettres> ...]\nexemple : squares AZER QSDF WXCV"
                .to_string(),
        );
    }

    let mut seen = HashSet::new();
    let mut groups = Vec::with_capacity(args.len());

    for arg in args {
        let chars: Vec<char> = arg.chars().collect();
        if chars.len() != 4 {
            return Err(format!(
                "chaque argument doit contenir exactement 4 lettres, reçu « {arg} » ({})",
                chars.len()
            ));
        }
        if !chars.iter().all(|c| c.is_ascii_alphabetic()) {
            return Err(format!("« {arg} » : seules les lettres A–Z sont autorisées"));
        }

        let keys: Vec<Key> = chars
            .iter()
            .map(|&c| key_from_char(c).ok_or_else(|| format!("touche invalide : {c}")))
            .collect::<Result<_, _>>()?;

        for &c in &chars {
            let upper = c.to_ascii_uppercase();
            if !seen.insert(upper) {
                return Err(format!("la touche « {upper} » est utilisée plusieurs fois"));
            }
        }

        let key_array: [Key; 4] = [keys[0], keys[1], keys[2], keys[3]];
        groups.push(controls_from_keys(key_array));
    }

    Ok(groups)
}

struct App {
    gl: GlGraphics,
    squares: Vec<Square>,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::{clear, rectangle};

        self.gl.draw(args.viewport(), |c, gl| {
            clear([0.12, 0.12, 0.16, 1.0], gl);

            for sq in &self.squares {
                let rect = rectangle::square(0.0, 0.0, SQUARE_SIZE);
                let transform = c.transform.trans(sq.x, sq.y);
                rectangle(sq.color, rect, transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        for sq in &mut self.squares {
            sq.step(args.dt);
        }
    }

    fn on_key(&mut self, key: Key, pressed: bool) {
        for sq in &mut self.squares {
            sq.apply_key(key, pressed);
        }
    }
}

fn main() {
    let controls = match parse_square_groups(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("Erreur : {msg}");
            process::exit(1);
        }
    };

    let total = controls.len();
    let squares: Vec<Square> = controls
        .into_iter()
        .enumerate()
        .map(|(i, ctrl)| Square::new(i, total, ctrl))
        .collect();

    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("squares", [800_u32, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .expect("impossible de créer la fenêtre");

    let mut app = App {
        gl: GlGraphics::new(opengl),
        squares,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            app.on_key(key, true);
        }
        if let Some(Button::Keyboard(key)) = e.release_args() {
            app.on_key(key, false);
        }
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
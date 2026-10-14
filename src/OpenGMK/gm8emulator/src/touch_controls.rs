//! On-screen touch controls for the Android port.
//!
//! The game only understands keyboard input, so each on-screen control injects the key the game
//! expects. The key set is the one the game binds by default:
//!
//! - arrows: move
//! - `Z` confirm/act, `X` cancel, `C`/`V` extra actions (fishing/bike, attacks 3/4)
//! - `1`..`6`: pick a party Pokémon in battle (and act as the HM shortcuts / berries)
//! - `B` pokéball, `S` switch, `M` mega, `E` run, `A` walk/assign, `N` use item
//! - `ENTER` menus, `ESC` quit
//!
//! Movement is an analog joystick (bottom-left); the rest are buttons. Positions are normalised to
//! the window so the input hit-test and the renderer share the exact same layout.
//!
//! Controls are grouped by the game's current state (see [current_context]): the title/menu rooms
//! only need the basics, the overworld needs the joystick and interaction keys, and battle needs
//! the combat extras. The emulator reports the current room every frame.

use ramen::input::Key;
use std::sync::{
    atomic::{AtomicI32, AtomicU32, Ordering},
    Mutex,
};

/// Which game states a control is shown in.
pub const MENU: u8 = 1;
pub const WORLD: u8 = 2;
pub const BATTLE: u8 = 4;
/// A dialogue box is open: only the button that advances it is shown, so nothing covers the text.
pub const DIALOG: u8 = 8;

#[derive(Clone, Copy, PartialEq)]
pub enum Shape {
    Circle { cx: f32, cy: f32, r: f32 },
    Rect { cx: f32, cy: f32, w: f32, h: f32 },
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Label {
    None,
    Arrow(Dir),
    Text(&'static str),
}

pub struct Control {
    /// Stable name, used as the key when the layout is saved and loaded. Reordering the array of
    /// controls must not move a player's customised button, so the name is what identifies it.
    pub name: &'static str,
    pub key: Key,
    pub shape: Shape,
    pub label: Label,
    /// Bitmask of [MENU] / [WORLD] / [BATTLE] states this control belongs to.
    pub contexts: u8,
}

/// The joystick is not a button but is customised (and saved) the same way.
pub const JOYSTICK_NAME: &str = "joystick";

pub const CONTROLS: &[Control] = &[
    // Party Pokémon 1-6 (battle: pick a Pokémon).
    Control { name: "alpha1", key: Key::Alpha1, shape: Shape::Circle { cx: 0.300, cy: 0.055, r: 0.020 }, label: Label::Text("1"), contexts: BATTLE },
    Control { name: "alpha2", key: Key::Alpha2, shape: Shape::Circle { cx: 0.372, cy: 0.055, r: 0.020 }, label: Label::Text("2"), contexts: BATTLE },
    Control { name: "alpha3", key: Key::Alpha3, shape: Shape::Circle { cx: 0.444, cy: 0.055, r: 0.020 }, label: Label::Text("3"), contexts: BATTLE },
    Control { name: "alpha4", key: Key::Alpha4, shape: Shape::Circle { cx: 0.516, cy: 0.055, r: 0.020 }, label: Label::Text("4"), contexts: BATTLE },
    Control { name: "alpha5", key: Key::Alpha5, shape: Shape::Circle { cx: 0.588, cy: 0.055, r: 0.020 }, label: Label::Text("5"), contexts: BATTLE },
    Control { name: "alpha6", key: Key::Alpha6, shape: Shape::Circle { cx: 0.660, cy: 0.055, r: 0.020 }, label: Label::Text("6"), contexts: BATTLE },
    // Menu / misc keys (second row along the top).
    Control { name: "space", key: Key::Space, shape: Shape::Circle { cx: 0.300, cy: 0.145, r: 0.022 }, label: Label::Text("PKM"), contexts: WORLD | BATTLE },
    Control { name: "o", key: Key::O, shape: Shape::Circle { cx: 0.372, cy: 0.145, r: 0.022 }, label: Label::Text("O"), contexts: WORLD | BATTLE },
    Control { name: "m", key: Key::M, shape: Shape::Circle { cx: 0.444, cy: 0.145, r: 0.022 }, label: Label::Text("M"), contexts: WORLD | BATTLE },
    Control { name: "e", key: Key::E, shape: Shape::Circle { cx: 0.516, cy: 0.145, r: 0.022 }, label: Label::Text("E"), contexts: WORLD | BATTLE },
    Control { name: "t", key: Key::T, shape: Shape::Circle { cx: 0.588, cy: 0.145, r: 0.022 }, label: Label::Text("T"), contexts: WORLD | BATTLE },
    Control { name: "l", key: Key::L, shape: Shape::Circle { cx: 0.660, cy: 0.145, r: 0.022 }, label: Label::Text("L"), contexts: WORLD | BATTLE },
    Control { name: "p", key: Key::P, shape: Shape::Circle { cx: 0.732, cy: 0.145, r: 0.022 }, label: Label::Text("P"), contexts: WORLD | BATTLE },
    // Main actions.
    Control { name: "z", key: Key::Z, shape: Shape::Circle { cx: 0.930, cy: 0.840, r: 0.034 }, label: Label::Text("Z"), contexts: WORLD | BATTLE },
    Control { name: "x", key: Key::X, shape: Shape::Circle { cx: 0.800, cy: 0.868, r: 0.031 }, label: Label::Text("X"), contexts: WORLD | BATTLE },
    Control { name: "c", key: Key::C, shape: Shape::Circle { cx: 0.858, cy: 0.635, r: 0.030 }, label: Label::Text("C"), contexts: WORLD | BATTLE },
    Control { name: "v", key: Key::V, shape: Shape::Circle { cx: 0.950, cy: 0.665, r: 0.028 }, label: Label::Text("V"), contexts: WORLD | BATTLE },
    // Secondary.
    Control { name: "b", key: Key::B, shape: Shape::Circle { cx: 0.712, cy: 0.840, r: 0.026 }, label: Label::Text("B"), contexts: BATTLE },
    Control { name: "s", key: Key::S, shape: Shape::Circle { cx: 0.712, cy: 0.700, r: 0.026 }, label: Label::Text("S"), contexts: BATTLE },
    Control { name: "a", key: Key::A, shape: Shape::Circle { cx: 0.640, cy: 0.900, r: 0.024 }, label: Label::Text("A"), contexts: WORLD },
    Control { name: "n", key: Key::N, shape: Shape::Circle { cx: 0.640, cy: 0.760, r: 0.024 }, label: Label::Text("N"), contexts: WORLD },
    // Menu keys.
    Control { name: "return", key: Key::Return, shape: Shape::Circle { cx: 0.500, cy: 0.945, r: 0.032 }, label: Label::Text("OK"), contexts: WORLD | BATTLE },
    Control { name: "escape", key: Key::Escape, shape: Shape::Circle { cx: 0.400, cy: 0.905, r: 0.026 }, label: Label::Text("ESC"), contexts: WORLD | BATTLE },
    // While a dialogue box is open the overlay steps aside and leaves only this, so the text and the
    // portrait are not buried under buttons. Its position is measured from the game image (not the
    // window), so it sits on the box whatever the screen's shape is. It sends the key the game
    // advances dialogues with.
    Control { name: "dialogo", key: Key::Z, shape: Shape::Circle { cx: 0.940, cy: 0.880, r: 0.024 }, label: Label::Text("OK"), contexts: DIALOG },
];

// ---------------------------------------------------------------------------------------------
// Customisable layout. Positions and sizes can be moved, resized and hidden by the player, both
// from the on-screen editor and from the launcher app; the two share `layout.conf`.

/// A player's adjustment to a control: where it sits (normalised 0..1), how big it is (a multiplier
/// on the built-in size) and whether it is shown.
#[derive(Clone, Copy)]
pub struct ControlOverride {
    pub cx: f32,
    pub cy: f32,
    pub scale: f32,
    pub visible: bool,
}

impl ControlOverride {
    fn of(shape: Shape) -> Self {
        let (cx, cy) = match shape {
            Shape::Circle { cx, cy, .. } | Shape::Rect { cx, cy, .. } => (cx, cy),
        };
        Self { cx, cy, scale: 1.0, visible: true }
    }
}

fn default_layout() -> Vec<ControlOverride> {
    CONTROLS.iter().map(|c| ControlOverride::of(c.shape)).collect()
}

fn default_joystick() -> ControlOverride {
    ControlOverride { cx: JOYSTICK_CX, cy: JOYSTICK_CY, scale: 1.0, visible: true }
}

/// `None` until first used, so the defaults live in one place (the `CONTROLS` table).
static LAYOUT: Mutex<Option<Vec<ControlOverride>>> = Mutex::new(None);
static JOYSTICK_LAYOUT: Mutex<Option<ControlOverride>> = Mutex::new(None);

fn with_layout<R>(f: impl FnOnce(&mut Vec<ControlOverride>) -> R) -> R {
    let mut guard = LAYOUT.lock().unwrap();
    if guard.is_none() {
        *guard = Some(default_layout());
    }
    f(guard.as_mut().unwrap())
}

fn with_joystick<R>(f: impl FnOnce(&mut ControlOverride) -> R) -> R {
    let mut guard = JOYSTICK_LAYOUT.lock().unwrap();
    if guard.is_none() {
        *guard = Some(default_joystick());
    }
    f(guard.as_mut().unwrap())
}

/// The control's shape with the player's position and size applied.
pub fn effective_shape(index: usize) -> Shape {
    let shape = CONTROLS[index].shape;
    with_layout(|layout| match shape {
        Shape::Circle { r, .. } => Shape::Circle {
            cx: layout[index].cx,
            cy: layout[index].cy,
            r: r * layout[index].scale,
        },
        Shape::Rect { w, h, .. } => Shape::Rect {
            cx: layout[index].cx,
            cy: layout[index].cy,
            w: w * layout[index].scale,
            h: h * layout[index].scale,
        },
    })
}

/// Name of the control, used to line the saved layout up with the table above.
pub fn control_name(index: usize) -> &'static str {
    CONTROLS[index].name
}

/// The rectangle the game image occupies in the window (normalised), as the renderer last presented
/// it. Dialogue controls are placed inside it, so the button lands on the dialogue box whatever the
/// screen's shape is and whichever scaling mode is in use.
static VIEWPORT: Mutex<(f32, f32, f32, f32)> = Mutex::new((0.0, 0.0, 1.0, 1.0));

pub fn set_viewport(x: f32, y: f32, w: f32, h: f32) {
    *VIEWPORT.lock().unwrap() = (x, y, w, h);
}

fn viewport() -> (f32, f32, f32, f32) {
    *VIEWPORT.lock().unwrap()
}

/// A control's shape in window coordinates. Most controls are placed relative to the window (they
/// live around its edges, where the thumbs are); the dialogue button is placed relative to the game
/// image, because it has to sit on the dialogue box the game draws.
pub fn window_shape(index: usize) -> Shape {
    let shape = effective_shape(index);
    if CONTROLS[index].contexts & DIALOG == 0 {
        return shape
    }
    let (vx, vy, vw, vh) = viewport();
    match shape {
        Shape::Circle { cx, cy, r } => Shape::Circle { cx: vx + cx * vw, cy: vy + cy * vh, r: r * vw },
        Shape::Rect { cx, cy, w, h } => Shape::Rect { cx: vx + cx * vw, cy: vy + cy * vh, w: w * vw, h: h * vh },
    }
}

/// The inverse: a point in window coordinates, expressed in a control's own coordinates.
pub fn control_point(index: usize, nx: f32, ny: f32) -> (f32, f32) {
    if CONTROLS[index].contexts & DIALOG == 0 {
        return (nx, ny)
    }
    let (vx, vy, vw, vh) = viewport();
    ((nx - vx) / vw.max(0.0001), (ny - vy) / vh.max(0.0001))
}

/// Moves a control, as normalised coordinates.
pub fn move_control(index: usize, cx: f32, cy: f32) {
    with_layout(|layout| {
        layout[index].cx = cx;
        layout[index].cy = cy;
    });
}

pub fn scale_control(index: usize, scale: f32) {
    with_layout(|layout| layout[index].scale = scale.max(0.3));
}

pub fn set_visible(index: usize, visible: bool) {
    with_layout(|layout| layout[index].visible = visible);
}

pub fn is_visible(index: usize) -> bool {
    with_layout(|layout| layout[index].visible)
}

/// The current layout, as `(name, override)` pairs, and the joystick's. Used to save the file.
pub fn snapshot() -> (Vec<(&'static str, ControlOverride)>, ControlOverride) {
    let controls = CONTROLS
        .iter()
        .enumerate()
        .map(|(index, control)| (control.name, with_layout(|layout| layout[index])))
        .collect();
    (controls, with_joystick(|joystick| *joystick))
}

/// Replaces the whole layout, e.g. after loading `layout.conf` or pressing "restore defaults".
pub fn set_layout(entries: &[(&'static str, ControlOverride)], joystick: Option<ControlOverride>) {
    with_layout(|layout| {
        for (index, control) in CONTROLS.iter().enumerate() {
            if let Some((_, value)) = entries.iter().find(|(name, _)| *name == control.name) {
                layout[index] = *value;
            } else {
                layout[index] = ControlOverride::of(control.shape);
            }
        }
    });
    with_joystick(|current| *current = joystick.unwrap_or_else(default_joystick));
}

pub fn restore_defaults() {
    with_layout(|layout| *layout = default_layout());
    with_joystick(|joystick| *joystick = default_joystick());
}

/// Looks a control up by its saved name.
pub fn index_of(name: &str) -> Option<usize> {
    CONTROLS.iter().position(|control| control.name == name)
}

fn parse_override(value: &str) -> Option<ControlOverride> {
    let mut parts = value.trim().split(',');
    let cx: f32 = parts.next()?.trim().parse().ok()?;
    let cy: f32 = parts.next()?.trim().parse().ok()?;
    let scale: f32 = parts.next()?.trim().parse().ok()?;
    let visible = matches!(parts.next()?.trim(), "1" | "true" | "si" | "sí");
    Some(ControlOverride { cx, cy, scale: scale.max(0.3), visible })
}

/// Reads a saved layout. Lines are `<name>=<cx>,<cy>,<scale>,<visible>`, with `joystick` as one of
/// the names. Anything unrecognised is ignored, so an old or hand-edited file never breaks the game.
pub fn parse_layout(text: &str) -> (Vec<(&'static str, ControlOverride)>, Option<ControlOverride>) {
    let mut controls = Vec::new();
    let mut joystick = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue
        }
        let Some((name, value)) = line.split_once('=') else { continue };
        let Some(parsed) = parse_override(value) else { continue };
        let name = name.trim().to_ascii_lowercase();
        if name == JOYSTICK_NAME {
            joystick = Some(parsed);
        } else if let Some(index) = index_of(&name) {
            controls.push((CONTROLS[index].name, parsed));
        }
    }
    (controls, joystick)
}

/// The layout file's contents for the current layout.
pub fn format_layout() -> String {
    let (controls, joystick) = snapshot();
    let mut text = String::from("# Layout de los controles tactiles; lo escriben el editor del juego y la app.\n");
    for (name, o) in controls {
        text.push_str(&format!("{}={:.4},{:.4},{:.4},{}\n", name, o.cx, o.cy, o.scale, u8::from(o.visible)));
    }
    text.push_str(&format!(
        "{}={:.4},{:.4},{:.4},{}\n",
        JOYSTICK_NAME,
        joystick.cx,
        joystick.cy,
        joystick.scale,
        u8::from(joystick.visible)
    ));
    text
}


/// The room the emulator reported, or -1 before the first frame.
static ROOM: AtomicI32 = AtomicI32::new(-1);

/// Called by the emulator every frame with the current room id.
pub fn set_room(room: i32) {
    ROOM.store(room, Ordering::Relaxed);
    log_state_change(room);
}

/// Whether the game has one of its dialogue boxes open. Reported every frame by the engine, which
/// reads it from the game's own modal state; the overlay then leaves a single button.
static DIALOGUE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_dialogue(open: bool) {
    let was = DIALOGUE.swap(open, Ordering::Relaxed);
    if was != open {
        eprintln!("[overlay] dialogo {}", if open { "abierto: solo el boton de avanzar" } else { "cerrado" });
    }
}

pub fn dialogue_open() -> bool {
    DIALOGUE.load(Ordering::Relaxed)
}

/// Records every change of state, which is what decides which on-screen controls exist.
///
/// A report of buttons missing, or in the wrong place, cannot be chased without knowing what the
/// overlay believed at the time: which room, which group of controls it picked, and how many of
/// them it counted as active. It only writes when something changes, so it stays quiet while
/// playing.
#[cfg(target_os = "android")]
fn log_state_change(room: i32) {
    static LAST: AtomicU32 = AtomicU32::new(u32::MAX);
    static LAST_ROOM: AtomicI32 = AtomicI32::new(i32::MIN);
    let group = u32::from(current_context());
    let room_changed = LAST_ROOM.swap(room, Ordering::Relaxed) != room;
    if LAST.swap(group, Ordering::Relaxed) == group {
        // The group did not change, but knowing which room we are in is what makes it possible to
        // say "this screen needs no controls" about a specific one.
        if room_changed {
            eprintln!("[overlay] sala {} -> grupo {}", room, group);
        }
        return
    }
    let (window_w, window_h) = ramen::platform::android::window_size();
    let active = (0..CONTROLS.len()).filter(|index| is_active(*index)).count();
    eprintln!("[overlay] sala {} -> grupo {} | ventana {}x{} | {} de {} activos", room, group, window_w, window_h, active, CONTROLS.len());
    // Every control, one per line, so a report of "buttons missing" can be checked against what the
    // overlay actually decided: which key, where it was put, how big it is, and whether it counted
    // as visible. Normalised coordinates are fractions of the window, so they can be compared
    // between devices of any size.
    for (index, control) in CONTROLS.iter().enumerate() {
        let (at, size) = match control.shape {
            Shape::Circle { cx, cy, r } => (format!("{:.3},{:.3}", cx, cy), format!("r={:.3}", r)),
            Shape::Rect { cx, cy, w, h } => (format!("{:.3},{:.3}", cx, cy), format!("{:.3}x{:.3}", w, h)),
        };
        let name = match control.label {
            Label::None => "",
            Label::Arrow(_) => "flecha",
            Label::Text(text) => text,
        };
        eprintln!(
            "[overlay]   {:>2}. {:<6} {:<12} en ({}) {} -> {}",
            index,
            name,
            format!("{:?}", control.key),
            at,
            size,
            if is_active(index) { "visibles" } else { "ocultos" }
        );
    }
}

/// Nothing to report where there is no overlay.
#[cfg(not(target_os = "android"))]
fn log_state_change(_room: i32) {}

/// Which group of controls the current game state needs.
pub fn current_context() -> u8 {
    // A dialogue box takes over the overlay: only the button that advances it is shown, so the text
    // and the portrait stay readable.
    if dialogue_open() {
        return DIALOG
    }
    match ROOM.load(Ordering::Relaxed) {
        // IntroROOM, InicioROOM, CargarPartida, MultijugadorMenu, CreditosInicioScreen, OptionsROOM:
        // these screens are fully clickable (once the game's own click boxes are fixed, see
        // `crate::patches`), so they need no on-screen controls at all.
        22 | 30 | 43 | 461 | 465 | 466 => MENU,
        // BattleROOM, BattleROOMAqua, BattleROOMLava.
        47 | 191 | 709 => BATTLE,
        // Before the first frame we don't know the room yet; show nothing rather than flashing the
        // whole overlay for one frame.
        -1 => MENU,
        _ => WORLD,
    }
}

/// True while the player is on a pad. The on-screen controls then step aside, because somebody
/// holding a controller does not want buttons under their thumbs; the first touch on the screen
/// brings them back.
static PAD_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_pad_active(active: bool) {
    PAD_ACTIVE.store(active, Ordering::Relaxed);
}

pub fn pad_active() -> bool {
    PAD_ACTIVE.load(Ordering::Relaxed)
}

pub fn is_active(index: usize) -> bool {
    !pad_active() && is_visible(index) && CONTROLS[index].contexts & current_context() != 0
}

/// The joystick drives the arrow keys, which is how the game's menus (the ones that still show
/// controls) and the overworld are navigated.
pub fn joystick_active() -> bool {
    !pad_active() && joystick_visible() && current_context() & (WORLD | BATTLE) != 0
}

/// Returns the index of the control containing the point, if any. The point is normalised, but the
/// test is done in pixels so the touchable area matches what is drawn on a screen that is not square.
pub fn hit(nx: f32, ny: f32, ww: f32, wh: f32) -> Option<usize> {
    CONTROLS
        .iter()
        .enumerate()
        .position(|(index, _)| is_active(index) && contains(window_shape(index), nx, ny, ww, wh))
}

static PRESSED: AtomicU32 = AtomicU32::new(0);

pub fn set_pressed(index: usize, pressed: bool) {
    if pressed {
        PRESSED.fetch_or(1 << index, Ordering::Relaxed);
    } else {
        PRESSED.fetch_and(!(1 << index), Ordering::Relaxed);
    }
}

pub fn is_pressed(index: usize) -> bool {
    PRESSED.load(Ordering::Relaxed) & (1 << index) != 0
}

/// Controls that are pressed but no longer valid, so the caller can release their keys instead of
/// leaving one stuck down when the game changes state.
pub fn stale_pressed() -> Vec<usize> {
    (0..CONTROLS.len()).filter(|&index| is_pressed(index) && !is_active(index)).collect()
}

// ---------------------------------------------------------------------------------------------
// Movement joystick. It presses the arrow keys depending on which way the stick is pushed.

pub const JOYSTICK_CX: f32 = 0.135;
pub const JOYSTICK_CY: f32 = 0.700;
/// Radius as a fraction of the window *height*, so the base stays round on screen.
pub const JOYSTICK_R: f32 = 0.200;
/// Dead zone, as a fraction of the radius, before a direction registers.
pub const JOYSTICK_DEAD: f32 = 0.35;

/// The joystick's centre and radius with the player's layout applied.
pub fn joystick_center() -> (f32, f32) {
    with_joystick(|joystick| (joystick.cx, joystick.cy))
}

pub fn joystick_radius() -> f32 {
    with_joystick(|joystick| JOYSTICK_R * joystick.scale)
}

pub fn joystick_visible() -> bool {
    with_joystick(|joystick| joystick.visible)
}

pub fn move_joystick(cx: f32, cy: f32) {
    with_joystick(|joystick| {
        joystick.cx = cx;
        joystick.cy = cy;
    });
}

pub fn scale_joystick(scale: f32) {
    with_joystick(|joystick| joystick.scale = scale.max(0.3));
}

pub fn set_joystick_visible(visible: bool) {
    with_joystick(|joystick| joystick.visible = visible);
}

pub fn joystick_hit(nx: f32, ny: f32, ww: f32, wh: f32) -> bool {
    if !joystick_active() {
        return false
    }
    let (cx, cy) = joystick_center();
    let r = joystick_radius() * wh;
    let dx = (nx - cx) * ww;
    let dy = (ny - cy) * wh;
    dx * dx + dy * dy <= r * r
}

/// `[up, down, left, right]` to hold for a stick at (nx, ny).
pub fn joystick_dirs(nx: f32, ny: f32, ww: f32, wh: f32) -> [bool; 4] {
    let (cx, cy) = joystick_center();
    let r = joystick_radius() * wh;
    let dx = (nx - cx) * ww;
    let dy = (ny - cy) * wh;
    stick_dirs(dx / r, dy / r)
}

/// Which directions a stick pushed by (dx, dy) is asking for: up, down, left, right. Both are in
/// the range -1..1, and this is the same dead zone the on-screen joystick uses - a physical pad's
/// stick goes through here too, so both feel identical.
pub fn stick_dirs(dx: f32, dy: f32) -> [bool; 4] {
    [
        dy < -JOYSTICK_DEAD,
        dy > JOYSTICK_DEAD,
        dx < -JOYSTICK_DEAD,
        dx > JOYSTICK_DEAD,
    ]
}

/// The stick's offset from its centre, in units of its radius (for drawing the knob).
static JOYSTICK_OFFSET: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

pub fn set_joystick_offset(dx: f32, dy: f32) {
    *JOYSTICK_OFFSET.lock().unwrap() = (dx, dy);
}

pub fn joystick_offset() -> (f32, f32) {
    *JOYSTICK_OFFSET.lock().unwrap()
}

// ---------------------------------------------------------------------------------------------
// On-screen editor. Entered from the gear button, it lets the player drag, resize and hide the
// controls right on the game, and saves the result to the same `layout.conf` the app edits.

/// The joystick, addressed like a control (one past the end of [CONTROLS]) while editing.
pub const JOYSTICK_INDEX: usize = CONTROLS.len();

pub const EDIT_MOVE: u8 = 0;
pub const EDIT_SIZE: u8 = 1;
pub const EDIT_VISIBILITY: u8 = 2;

/// What a press in the editor is asking the caller to do.
pub enum EditorAction {
    None,
    Save,
    Restore,
    Exit,
}

static EDITOR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static EDIT_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(EDIT_MOVE);
static EDIT_SELECTED: AtomicI32 = AtomicI32::new(-1);
/// Where the finger grabbed the control, so dragging does not make it jump under the thumb.
static EDIT_GRAB: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

pub fn editor_active() -> bool {
    EDITOR.load(Ordering::Relaxed)
}

pub fn set_editor_active(active: bool) {
    EDITOR.store(active, Ordering::Relaxed);
    EDIT_SELECTED.store(-1, Ordering::Relaxed);
}

pub fn edit_mode() -> u8 {
    EDIT_MODE.load(Ordering::Relaxed)
}

pub fn set_edit_mode(mode: u8) {
    EDIT_MODE.store(mode, Ordering::Relaxed);
}

/// The index being edited, or -1.
pub fn editor_selected() -> i32 {
    EDIT_SELECTED.load(Ordering::Relaxed)
}

/// The gear button, top-right corner (normalised).
pub fn gear_shape() -> Shape {
    Shape::Circle { cx: 0.962, cy: 0.070, r: 0.030 }
}

/// The editor's bottom bar: three modes and three actions.
pub const TOOLBAR_LABELS: [&str; 6] = ["MOVER", "TAMANO", "OCULTAR", "GUARDAR", "RESET", "SALIR"];

pub fn toolbar_shape(index: usize) -> Shape {
    let w = 1.0 / TOOLBAR_LABELS.len() as f32;
    Shape::Rect { cx: w * (index as f32 + 0.5), cy: 0.958, w: w * 0.94, h: 0.072 }
}

fn center_of(shape: Shape) -> (f32, f32) {
    match shape {
        Shape::Circle { cx, cy, .. } | Shape::Rect { cx, cy, .. } => (cx, cy),
    }
}

/// Point-in-shape, in pixels so it matches what is drawn.
pub fn contains(shape: Shape, nx: f32, ny: f32, ww: f32, wh: f32) -> bool {
    match shape {
        Shape::Circle { cx, cy, r } => {
            let (dx, dy) = ((nx - cx) * ww, (ny - cy) * wh);
            let radius = r * ww;
            dx * dx + dy * dy <= radius * radius
        },
        Shape::Rect { cx, cy, w, h } => (nx - cx).abs() * ww <= w * ww / 2.0 && (ny - cy).abs() * wh <= h * wh / 2.0,
    }
}

/// Which control (or the joystick) is under the point, ignoring visibility and context.
fn editor_target(nx: f32, ny: f32, ww: f32, wh: f32) -> i32 {
    for index in 0..CONTROLS.len() {
        if contains(window_shape(index), nx, ny, ww, wh) {
            return index as i32
        }
    }
    if contains(Shape::Circle { cx: joystick_center().0, cy: joystick_center().1, r: joystick_radius() }, nx, ny, ww, wh) {
        return JOYSTICK_INDEX as i32
    }
    -1
}

/// Handles a press while the editor is open.
pub fn editor_press(nx: f32, ny: f32, ww: f32, wh: f32) -> EditorAction {
    for index in 0..TOOLBAR_LABELS.len() {
        if contains(toolbar_shape(index), nx, ny, ww, wh) {
            match index {
                0 => set_edit_mode(EDIT_MOVE),
                1 => set_edit_mode(EDIT_SIZE),
                2 => set_edit_mode(EDIT_VISIBILITY),
                3 => return EditorAction::Save,
                4 => {
                    restore_defaults();
                    // The caller has to write the file, or the old layout comes back as soon as it
                    // is read again.
                    return EditorAction::Restore
                },
                _ => return EditorAction::Exit,
            }
            return EditorAction::None
        }
    }
    if contains(gear_shape(), nx, ny, ww, wh) {
        return EditorAction::Exit
    }
    let target = editor_target(nx, ny, ww, wh);
    EDIT_SELECTED.store(target, Ordering::Relaxed);
    if target >= 0 {
        let (cx, cy) = if target == JOYSTICK_INDEX as i32 {
            joystick_center()
        } else {
            center_of(window_shape(target as usize))
        };
        *EDIT_GRAB.lock().unwrap() = (nx - cx, ny - cy);
        if edit_mode() == EDIT_VISIBILITY {
            if target == JOYSTICK_INDEX as i32 {
                set_joystick_visible(!joystick_visible());
            } else {
                set_visible(target as usize, !is_visible(target as usize));
            }
        }
    }
    EditorAction::None
}

/// Handles a drag while the editor is open: moves the selected control, or resizes it.
pub fn editor_move(nx: f32, ny: f32) {
    let target = EDIT_SELECTED.load(Ordering::Relaxed);
    if target < 0 {
        return
    }
    let joystick = target == JOYSTICK_INDEX as i32;
    let (gx, gy) = *EDIT_GRAB.lock().unwrap();
    match edit_mode() {
        EDIT_MOVE => {
            let (wx, wy) = (nx - gx, ny - gy);
            if joystick {
                move_joystick(wx, wy);
            } else {
                // Dialogue controls measure from the game image, so the finger is converted first.
                let (cx, cy) = control_point(target as usize, wx, wy);
                move_control(target as usize, cx, cy);
            }
        },
        EDIT_SIZE => {
            let (px, py) = if joystick { (nx, ny) } else { control_point(target as usize, nx, ny) };
            let (cx, cy) = if joystick { joystick_center() } else { center_of(effective_shape(target as usize)) };
            let distance = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
            // A drag about a third of the screen out roughly doubles the size.
            let scale = (distance / 0.15).clamp(0.3, 3.0);
            if joystick {
                scale_joystick(scale);
            } else {
                scale_control(target as usize, scale);
            }
        },
        _ => (),
    }
}

pub fn editor_release() {}

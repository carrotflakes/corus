use std::path::Path;

pub trait Root: 'static {
    type Ui: Ui;
    type VideoSubsystem: VideoSubsystem;
    type AudioSubsystem: AudioSubsystem;
    type Window: Window;
    type Canvas: Canvas;
    type Font: Font;
}

pub trait Ui: 'static + Sized {
    type VideoSubsystem: VideoSubsystem;
    type AudioSubsystem: AudioSubsystem;

    fn init() -> Result<Self, String>;
    fn video(&mut self) -> Result<Self::VideoSubsystem, String>;
    fn audio(&mut self) -> Result<Self::AudioSubsystem, String>;
    fn load_font(&mut self, path: impl AsRef<Path>, point_size: u16) -> Result<<<<Self::VideoSubsystem as VideoSubsystem>::Window as Window>::Canvas as Canvas>::Font, String>;
    fn events(&mut self) -> Vec<Event>;
}

pub trait VideoSubsystem {
    type Window: Window;

    fn new_window(&mut self, title: &str, width: u32, height: u32) -> Result<Self::Window, String>;
    fn text_input_start(&mut self, rect: Rect);
}

pub trait AudioSubsystem {
    fn open_playback<A: AudioCallback>(&mut self, get_callback: &mut dyn FnMut(i32) -> A) -> Result<Box<dyn AudioDevice<A>>, String>;
}

pub trait AudioCallback: Send + Sync + 'static {
    fn callback(&mut self, out: &mut [f32]);
}

pub trait AudioDevice<A: AudioCallback> {
}

pub trait Window {
    type Canvas: Canvas;

    fn into_canvas(self) -> Result<Self::Canvas, String>;
}

pub trait Canvas {
    type Font: Font;

    fn draw_line(&mut self, start: Point, end: Point) -> Result<(), String>;
    fn set_draw_color(&mut self, rgb: RGB);
    fn draw_rect(&mut self, rect: Rect);
    fn draw_text(&mut self, font: &mut Self::Font, text: &str, x: i32, y: i32) -> (u32, u32);
    fn clear(&mut self);
    fn present(&mut self);
}

pub trait Font {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point(pub i32, pub i32);

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point(x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect(pub i32, pub i32, pub u32, pub u32);

impl Rect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Rect(x, y, w, h)
    }

    pub fn contains_point(&self, point: Point) -> bool {
        self.0 <= point.0 && self.1 <= point.1 && point.0 < self.0 + self.2 as i32 && point.1 < self.1 + self.3 as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RGB(pub u8, pub u8, pub u8);

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Quit {
        timestamp: u32,
    },
    // AppTerminating {
    //     timestamp: u32,
    // },
    // AppLowMemory {
    //     timestamp: u32,
    // },
    // AppWillEnterBackground {
    //     timestamp: u32,
    // },
    // AppDidEnterBackground {
    //     timestamp: u32,
    // },
    // AppWillEnterForeground {
    //     timestamp: u32,
    // },
    // AppDidEnterForeground {
    //     timestamp: u32,
    // },
    KeyDown {
        timestamp: u32,
        window_id: u32,
        keycode: Option<Keycode>,
        // scancode: Option<Scancode>,
        // keymod: Mod,
        repeat: bool,
    },
    KeyUp {
        timestamp: u32,
        window_id: u32,
        keycode: Option<Keycode>,
        // scancode: Option<Scancode>,
        // keymod: Mod,
        repeat: bool,
    },
    TextEditing {
        timestamp: u32,
        window_id: u32,
        text: String,
        start: i32,
        length: i32,
    },
    TextInput {
        timestamp: u32,
        window_id: u32,
        text: String,
    },
    MouseMotion {
        timestamp: u32,
        window_id: u32,
        which: u32,
        // mousestate: MouseState,
        x: i32,
        y: i32,
        xrel: i32,
        yrel: i32,
    },
    MouseButtonDown {
        timestamp: u32,
        window_id: u32,
        which: u32,
        mouse_btn: MouseButton,
        clicks: u8,
        x: i32,
        y: i32,
    },
    MouseButtonUp {
        timestamp: u32,
        window_id: u32,
        which: u32,
        mouse_btn: MouseButton,
        clicks: u8,
        x: i32,
        y: i32,
    },
    MouseWheel {
        timestamp: u32,
        window_id: u32,
        which: u32,
        x: i32,
        y: i32,
        // direction: MouseWheelDirection,
    },
    // ControllerAxisMotion {
    //     timestamp: u32,
    //     which: u32,
    //     // axis: Axis,
    //     value: i16,
    // },
    // ControllerButtonDown {
    //     timestamp: u32,
    //     which: u32,
    //     // button: Button,
    // },
    // ControllerButtonUp {
    //     timestamp: u32,
    //     which: u32,
    //     // button: Button,
    // },
    // ControllerDeviceAdded {
    //     timestamp: u32,
    //     which: u32,
    // },
    // ControllerDeviceRemoved {
    //     timestamp: u32,
    //     which: u32,
    // },
    // ControllerDeviceRemapped {
    //     timestamp: u32,
    //     which: u32,
    // },
    FingerDown {
        timestamp: u32,
        touch_id: i64,
        finger_id: i64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },
    FingerUp {
        timestamp: u32,
        touch_id: i64,
        finger_id: i64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },
    FingerMotion {
        timestamp: u32,
        touch_id: i64,
        finger_id: i64,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
        pressure: f32,
    },
    // DollarGesture {
    //     timestamp: u32,
    //     touch_id: i64,
    //     gesture_id: i64,
    //     num_fingers: u32,
    //     error: f32,
    //     x: f32,
    //     y: f32,
    // },
    // DollarRecord {
    //     timestamp: u32,
    //     touch_id: i64,
    //     gesture_id: i64,
    //     num_fingers: u32,
    //     error: f32,
    //     x: f32,
    //     y: f32,
    // },
    // MultiGesture {
    //     timestamp: u32,
    //     touch_id: i64,
    //     d_theta: f32,
    //     d_dist: f32,
    //     x: f32,
    //     y: f32,
    //     num_fingers: u16,
    // },
    // ClipboardUpdate {
    //     timestamp: u32,
    // },
    // DropFile {
    //     timestamp: u32,
    //     window_id: u32,
    //     filename: String,
    // },
    // DropText {
    //     timestamp: u32,
    //     window_id: u32,
    //     filename: String,
    // },
    // DropBegin {
    //     timestamp: u32,
    //     window_id: u32,
    // },
    // DropComplete {
    //     timestamp: u32,
    //     window_id: u32,
    // },
    // AudioDeviceAdded {
    //     timestamp: u32,
    //     which: u32,
    //     iscapture: bool,
    // },
    // AudioDeviceRemoved {
    //     timestamp: u32,
    //     which: u32,
    //     iscapture: bool,
    // },
    // RenderTargetsReset {
    //     timestamp: u32,
    // },
    // RenderDeviceReset {
    //     timestamp: u32,
    // },
    // User {
    //     timestamp: u32,
    //     window_id: u32,
    //     type_: u32,
    //     code: i32,
    // },
    // Unknown {
    //     timestamp: u32,
    //     type_: u32,
    // },
    Unknown
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keycode {
    Backspace,
    Tab,
    Return,
    Escape,
    Space,
    Exclaim,
    Quotedbl,
    Hash,
    Dollar,
    Percent,
    Ampersand,
    Quote,
    LeftParen,
    RightParen,
    Asterisk,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Colon,
    Semicolon,
    Less,
    Equals,
    Greater,
    Question,
    At,
    LeftBracket,
    Backslash,
    RightBracket,
    Caret,
    Underscore,
    Backquote,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Delete,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    End,
    PageDown,
    Right,
    Left,
    Down,
    Up,
    NumLockClear,
    KpDivide,
    KpMultiply,
    KpMinus,
    KpPlus,
    KpEnter,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    Kp0,
    KpPeriod,
    Application,
    Power,
    KpEquals,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Execute,
    Help,
    Menu,
    Select,
    Stop,
    Again,
    Undo,
    Cut,
    Copy,
    Paste,
    Find,
    Mute,
    VolumeUp,
    VolumeDown,
    KpComma,
    KpEqualsAS400,
    AltErase,
    Sysreq,
    Cancel,
    Clear,
    Prior,
    Return2,
    Separator,
    Out,
    Oper,
    ClearAgain,
    CrSel,
    ExSel,
    Kp00,
    Kp000,
    ThousandsSeparator,
    DecimalSeparator,
    CurrencyUnit,
    CurrencySubUnit,
    KpLeftParen,
    KpRightParen,
    KpLeftBrace,
    KpRightBrace,
    KpTab,
    KpBackspace,
    KpA,
    KpB,
    KpC,
    KpD,
    KpE,
    KpF,
    KpXor,
    KpPower,
    KpPercent,
    KpLess,
    KpGreater,
    KpAmpersand,
    KpDblAmpersand,
    KpVerticalBar,
    KpDblVerticalBar,
    KpColon,
    KpHash,
    KpSpace,
    KpAt,
    KpExclam,
    KpMemStore,
    KpMemRecall,
    KpMemClear,
    KpMemAdd,
    KpMemSubtract,
    KpMemMultiply,
    KpMemDivide,
    KpPlusMinus,
    KpClear,
    KpClearEntry,
    KpBinary,
    KpOctal,
    KpDecimal,
    KpHexadecimal,
    LCtrl,
    LShift,
    LAlt,
    LGui,
    RCtrl,
    RShift,
    RAlt,
    RGui,
    Mode,
    AudioNext,
    AudioPrev,
    AudioStop,
    AudioPlay,
    AudioMute,
    MediaSelect,
    Www,
    Mail,
    Calculator,
    Computer,
    AcSearch,
    AcHome,
    AcBack,
    AcForward,
    AcStop,
    AcRefresh,
    AcBookmarks,
    BrightnessDown,
    BrightnessUp,
    DisplaySwitch,
    KbdIllumToggle,
    KbdIllumDown,
    KbdIllumUp,
    Eject,
    Sleep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Unknown,
    Left,
    Middle,
    Right,
    X1,
    X2,
}

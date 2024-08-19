use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
    time::Duration,
};

use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    event::Event,
    keyboard::Scancode,
    pixels::Color,
    rect::Rect,
    render::WindowCanvas,
};

/// Beep sound.
///
/// This should be played when the sound register is non-zero.
struct Beep {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for Beep {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            // Generate a simple square wave
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WindowCommand {
    Draw,
    WaitKeyPress,
    IsPressed(u8),
    Clear,
    ControlSound(bool),
}

#[derive(Debug)]
pub struct Window {
    /// Fame Buffer of the current window.
    ///
    /// Since the interpreter only supports a max width of 64 pixel,
    /// `u64`s are (mis-)used as bitfields.
    frame_buffer: Arc<RwLock<[u64; Self::HEIGHT]>>,
    sender: Option<std::sync::mpsc::Sender<WindowCommand>>,
    receiver: Option<std::sync::mpsc::Receiver<u8>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Window {
    pub fn new(frame_buffer: Arc<RwLock<[u64; Self::HEIGHT]>>) -> Self {
        Self {
            frame_buffer,
            sender: None,
            receiver: None,
            thread: None,
        }
    }

    /// Width of the interpreter window.
    pub const WIDTH: usize = 64;

    /// Height of the interpreter window.
    pub const HEIGHT: usize = 32;

    /// Scale factor, which each pixel is scaled by.
    const SCALE_FACTOR: usize = 10;

    /// Color of the background (non-lit pixels) of the window
    const COLOR_BACKGROUND: Color = Color::RGB(28, 29, 30);

    /// Color of the foreground (lit pixels) of the window
    const COLOR_FOREGROUND: Color = Color::RGB(182, 236, 170);

    /// Digits that the interpreter can display.
    /// Ordered from 0 to F.
    pub const DIGITS: [[u8; 5]; 16] = [
        [0xF0, 0x90, 0x90, 0x90, 0xF0],
        [0x20, 0x60, 0x20, 0x20, 0x70],
        [0xF0, 0x10, 0xF0, 0x80, 0xF0],
        [0xF0, 0x10, 0xF0, 0x10, 0xF0],
        [0x90, 0x90, 0xF0, 0x10, 0x10],
        [0xF0, 0x80, 0xF0, 0x10, 0xF0],
        [0xF0, 0x80, 0xF0, 0x90, 0xF0],
        [0xF0, 0x10, 0x20, 0x40, 0x40],
        [0xF0, 0x90, 0xF0, 0x90, 0xF0],
        [0xF0, 0x90, 0xF0, 0x10, 0xF0],
        [0xF0, 0x90, 0xF0, 0x90, 0x90],
        [0xE0, 0x90, 0xE0, 0x90, 0xE0],
        [0xF0, 0x80, 0x80, 0x80, 0xF0],
        [0xE0, 0x90, 0x90, 0x90, 0xE0],
        [0xF0, 0x80, 0xF0, 0x80, 0xF0],
        [0xF0, 0x80, 0xF0, 0x80, 0x80],
    ];

    /// Queues a call.
    /// This causes the window contents to be redrawn, based on the [`Self::frame_buffer`].
    pub fn queue_draw(&self) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };
        let _ = sender.send(WindowCommand::Draw);
        // due to waiting for an interupt, the CHIP-8 is limited to 60 fps
        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }

    /// Clears the current screen.
    pub fn clear(&self) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };
        let _ = sender.send(WindowCommand::Clear);
        // reset the frame_buffer to 0
        let mut frame_buffer = self.frame_buffer.write().unwrap();
        frame_buffer.fill(0);
    }

    /// Checks if the given key is pressed.
    pub fn is_key_pressed(&mut self, key: u8) -> bool {
        let Some(sender) = self.sender.as_ref() else {
            return false;
        };
        sender.send(WindowCommand::IsPressed(key)).unwrap();
        match self.receiver.as_ref().unwrap().recv() {
            Ok(val) => val != 0,
            Err(_) => {
                eprintln!("Failed to receive reponse");
                false
            }
        }
    }

    /// Checks if the given key is pressed.
    pub fn wait_for_key_press(&mut self) -> u8 {
        self.sender
            .as_ref()
            .unwrap()
            .send(WindowCommand::WaitKeyPress)
            .unwrap();
        match self.receiver.as_ref().unwrap().recv() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Failed to receive reponse");
                0
            }
        }
    }

    /// Controls the sound.
    ///
    /// If `playing` is set to `true`, a constant beep is emitted.
    pub fn control_sound(&self, playing: bool) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };
        let _ = sender.send(WindowCommand::ControlSound(playing));
    }

    /// Checks if the window is still open
    pub fn is_open(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|handle| !handle.is_finished())
    }

    pub fn spawn(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel::<WindowCommand>();
        let (respond_tx, respond_rx) = std::sync::mpsc::channel::<u8>();
        self.sender.replace(tx);
        self.receiver.replace(respond_rx);
        let frame_buffer = Arc::clone(&self.frame_buffer);
        self.thread.replace(std::thread::spawn(move || {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let audio_subsystem = sdl_context.audio().unwrap();

            let audio = audio_subsystem
                .open_playback(
                    None,
                    &(AudioSpecDesired {
                        freq: Some(44100),
                        channels: Some(1),
                        samples: Some(4096),
                    }),
                    |spec| Beep {
                        phase_inc: 440.0 / spec.freq as f32,
                        phase: 0.0,
                        volume: 0.25,
                    },
                )
                .unwrap();

            let window = video_subsystem
                .window(
                    "CHIP-8 Emulator",
                    (Self::WIDTH * Self::SCALE_FACTOR) as u32,
                    (Self::HEIGHT * Self::SCALE_FACTOR) as u32,
                )
                .position_centered()
                .vulkan()
                .build()
                .unwrap();

            let mut canvas = window
                .into_canvas()
                .build()
                .map_err(|e| e.to_string())
                .unwrap();
            canvas.set_draw_color(Self::COLOR_BACKGROUND);
            canvas.present();
            let mut event_pump = sdl_context.event_pump().unwrap();

            let mut wait_for_key = false;
            loop {
                match rx.recv_timeout(std::time::Duration::new(0, 1_000_000_000u32 / 30)) {
                    Ok(WindowCommand::Draw) => Self::draw(&frame_buffer, &mut canvas),
                    Ok(WindowCommand::Clear) => {
                        canvas.set_draw_color(Self::COLOR_BACKGROUND);
                        canvas.clear();
                    }
                    Ok(WindowCommand::IsPressed(key)) => {
                        respond_tx
                            .send(
                                event_pump
                                    .keyboard_state()
                                    .is_scancode_pressed(Self::map_key(key))
                                    as u8,
                            )
                            .expect("Failed to send keycode");
                    }
                    Ok(WindowCommand::WaitKeyPress) => {
                        wait_for_key = true;
                    }
                    Ok(WindowCommand::ControlSound(true)) => audio.resume(),
                    Ok(WindowCommand::ControlSound(false)) => audio.pause(),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(_err) => {
                        eprintln!("Receiver died; quitting window");
                        return;
                    }
                };

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => return,
                        Event::KeyUp {
                            scancode: Some(key),
                            ..
                        } if wait_for_key => {
                            if let Some(mapped_key) = Self::map_scancode(key) {
                                respond_tx.send(mapped_key).expect("Failed to send keycode");
                                wait_for_key = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }));
    }

    /// Draws the screen based on the cucrrent [`Self::frame_buffer`].
    fn draw(frame_buffer: &Arc<RwLock<[u64; Self::HEIGHT]>>, canvas: &mut WindowCanvas) {
        let frame_buffer = frame_buffer.read().unwrap();
        // clear screen
        canvas.set_draw_color(Self::COLOR_BACKGROUND);
        canvas.clear();

        // draw new screen
        canvas.set_draw_color(Self::COLOR_FOREGROUND);
        for y in 0..Self::HEIGHT {
            for x in 0..Self::WIDTH {
                if (frame_buffer[y] & (1 << (Self::WIDTH - 1 - x))) == 0 {
                    continue;
                }
                canvas
                    .fill_rect(Rect::new(
                        (x * Self::SCALE_FACTOR) as i32,
                        (y * Self::SCALE_FACTOR) as i32,
                        Self::SCALE_FACTOR as u32,
                        Self::SCALE_FACTOR as u32,
                    ))
                    .expect("Failed to draw rect");
            }
        }
        canvas.present();
    }

    /// Maps a scancode the an CHIP-8 key.
    ///
    /// The scancodes are mapped as follows:
    /// Keypad       Keyboard
    /// +-+-+-+-+    +-+-+-+-+
    /// |1|2|3|C|    |1|2|3|4|
    /// +-+-+-+-+    +-+-+-+-+
    /// |4|5|6|D|    |Q|W|E|R|
    /// +-+-+-+-+ => +-+-+-+-+
    /// |7|8|9|E|    |A|S|D|F|
    /// +-+-+-+-+    +-+-+-+-+
    /// |A|0|B|F|    |Z|X|C|V|
    /// +-+-+-+-+    +-+-+-+-+
    fn map_scancode(key: Scancode) -> Option<u8> {
        Some(match key {
            Scancode::Num1 => 0x1,
            Scancode::Num2 => 0x2,
            Scancode::Num3 => 0x3,
            Scancode::Num4 => 0xC,
            Scancode::Q => 0x4,
            Scancode::W => 0x5,
            Scancode::E => 0x6,
            Scancode::R => 0xD,
            Scancode::A => 0x7,
            Scancode::S => 0x8,
            Scancode::D => 0x9,
            Scancode::F => 0xE,
            Scancode::Z => 0xA,
            Scancode::X => 0x0,
            Scancode::C => 0xB,
            Scancode::V => 0xF,
            _ => return None,
        })
    }

    /// Maps a CHIP-8 key to a physical scancode.
    ///
    /// The keys are mapped as follows:
    /// Keypad       Keyboard
    /// +-+-+-+-+    +-+-+-+-+
    /// |1|2|3|4|    |1|2|3|C|
    /// +-+-+-+-+    +-+-+-+-+
    /// |Q|W|E|R|    |4|5|6|D|
    /// +-+-+-+-+ => +-+-+-+-+
    /// |A|S|D|F|    |7|8|9|E|
    /// +-+-+-+-+    +-+-+-+-+
    /// |Z|X|C|V|    |A|0|B|F|
    /// +-+-+-+-+    +-+-+-+-+
    fn map_key(key: u8) -> Scancode {
        match key {
            0x1 => Scancode::Num1,
            0x2 => Scancode::Num2,
            0x3 => Scancode::Num3,
            0xC => Scancode::Num4,
            0x4 => Scancode::Q,
            0x5 => Scancode::W,
            0x6 => Scancode::E,
            0xD => Scancode::R,
            0x7 => Scancode::A,
            0x8 => Scancode::S,
            0x9 => Scancode::D,
            0xE => Scancode::F,
            0xA => Scancode::Z,
            0x0 => Scancode::X,
            0xB => Scancode::C,
            0xF => Scancode::V,
            _ => unreachable!("Trying to map invalid key {key}"),
        }
    }
}

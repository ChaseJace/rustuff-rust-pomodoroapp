use eframe::egui;
use rodio::source::SineWave;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy)]
enum TimerMode {
    Work,
    Rest,
}

struct PomodoroApp {
    work_minutes: u64,
    rest_minutes: u64,
    seconds_remaining: u64,
    current_mode: TimerMode,
    is_running: bool,
    last_tick: Option<Instant>,
    // Audio handles required to keep the sound engine alive
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl Default for PomodoroApp {
    fn default() -> Self {
        let default_work = 25;

        // Initialize the default audio output device stream
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to find a default audio output device");

        Self {
            work_minutes: default_work,
            rest_minutes: 5,
            seconds_remaining: default_work * 60,
            current_mode: TimerMode::Work,
            is_running: false,
            last_tick: None,
            _stream,
            stream_handle,
        }
    }
}

impl PomodoroApp {
    fn set_preset(&mut self, work: u64, rest: u64) {
        self.work_minutes = work;
        self.rest_minutes = rest;
        self.is_running = false;
        self.reset_timer();
    }

    fn reset_timer(&mut self) {
        self.seconds_remaining = match self.current_mode {
            TimerMode::Work => self.work_minutes * 60,
            TimerMode::Rest => self.rest_minutes * 60,
        };
        self.last_tick = None;
    }

    // Helper function to play a synthesized audio beep
    fn play_beep(&self, frequency: f32, duration_ms: u64) {
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            let source = SineWave::new(frequency)
                .take_duration(Duration::from_millis(duration_ms))
                .amplify(0.15); // Set volume to 15% so it's not too loud
            sink.append(source);
            sink.detach(); // Detach the sink so it plays independently in the background
        }
    }
}

impl eframe::App for PomodoroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_running {
            ctx.request_repaint_after(Duration::from_secs(1));

            match self.last_tick {
                None => {
                    self.last_tick = Some(Instant::now());
                }
                Some(last) => {
                    if last.elapsed() >= Duration::from_secs(1) {
                        if self.seconds_remaining > 0 {
                            self.seconds_remaining -= 1;
                        } else {
                            // Time's up! Play a cheerful chime (C5 note = 523.25 Hz)
                            self.play_beep(523.25, 500);

                            self.current_mode = match self.current_mode {
                                TimerMode::Work => TimerMode::Rest,
                                TimerMode::Rest => TimerMode::Work,
                            };
                            self.reset_timer();
                        }
                        self.last_tick = Some(Instant::now());
                    }
                }
            }
        }

        // --- Render UI Layout ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Pomodoro Timer");
            ui.add_space(15.0);

            let mode_str = match self.current_mode {
                TimerMode::Work => "🧠 Work Session",
                TimerMode::Rest => "☕ Rest Session",
            };
            ui.label(egui::RichText::new(mode_str).size(20.0).strong());

            ui.add_space(10.0);

            let minutes = self.seconds_remaining / 60;
            let seconds = self.seconds_remaining % 60;
            let time_display = format!("{:02}:{:02}", minutes, seconds);

            ui.label(egui::RichText::new(time_display).size(48.0).strong());

            ui.add_space(15.0);

            ui.horizontal(|ui| {
                if self.is_running {
                    if ui.button("⏸ Pause").clicked() {
                        self.is_running = false;
                        self.last_tick = None;
                    }
                } else {
                    if ui.button("▶ Start").clicked() {
                        self.is_running = true;
                        self.last_tick = Some(Instant::now());
                        // Optional: Play a short low beep on start
                        self.play_beep(440.0, 200);
                    }
                }

                if ui.button("🔄 Reset").clicked() {
                    self.is_running = false;
                    self.reset_timer();
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label("Quick Presets:");
            ui.horizontal(|ui| {
                if ui.button("25 / 5 min").clicked() {
                    self.set_preset(25, 5);
                }
                if ui.button("30 / 7 min").clicked() {
                    self.set_preset(30, 7);
                }
                if ui.button("60 / 10 min").clicked() {
                    self.set_preset(60, 10);
                }
                if ui.button("1 / 1 min").clicked() {
                    self.set_preset(1, 1);
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pomodoro Timer GUI",
        options,
        Box::new(|_cc| Ok(Box::new(PomodoroApp::default()) as Box<dyn eframe::App>)),
    )
}

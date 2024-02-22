use bevy::log::LogPlugin;

pub fn log_plugin() -> LogPlugin {
    let mut log_config = LogPlugin::default();
    log_config.filter.push_str(",botgame=debug");
    log_config
}

#[cfg(feature = "trace")]
pub mod tracing {
    use std::io::Write;

    use bevy::{input::common_conditions::input_just_pressed, prelude::*};
    use tracing_chrome::FlushGuard;

    pub fn tracing_plugin(app: &mut App) {
        app.add_systems(PreStartup, init_trace);
        app.add_systems(PreUpdate, new_trace.run_if(input_just_pressed(KeyCode::KeyL)));
    }

    fn init_trace(flush_guard: NonSend<FlushGuard>) {
        info!("Sending tracing data to the void.");
        flush_guard.start_new(Some(Box::new(Void)))
    }

    fn new_trace(mut tracing: Local<bool>, flush_guard: NonSend<FlushGuard>) {
        let writer = if *tracing {
            info!("Stopping detailed trace");
            Box::new(Void) as Box<dyn Write + Send>
        } else {
            let filename = format!(
                "./trace-{}.json",
                std::time::SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs()
            );
            info!("Starting detailed trace: {filename}");
            let Ok(fd) = std::fs::File::create(&filename) else {
                error!("Failed to create trace file: {filename}");
                return;
            };
            Box::new(fd) as Box<dyn std::io::Write + Send>
        };
        *tracing = !*tracing;
        flush_guard.start_new(Some(writer))
    }

    struct Void;
    impl Write for Void {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}

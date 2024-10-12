// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::error::Error;
use std::{
    io::stdin,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    const KEY_PRESS_INTERVAL: u8 = 5;
    let one_second = Duration::from_secs(1);
    let is_app_running = Arc::new(Mutex::new(true));
    let is_running = Arc::new(Mutex::new(true));

    {
        ui.on_request_start({
            let ui_handle = ui.as_weak();
            let is_running = Arc::clone(&is_running);
            move || {
                let ui = ui_handle.unwrap();
                ui.set_status(true);
                *is_running.lock().unwrap() = true;
            }
        });
    }

    {
        ui.on_request_stop({
            let ui_handle = ui.as_weak();
            let is_running = Arc::clone(&is_running);
            move || {
                let ui = ui_handle.unwrap();
                ui.set_status(false);
                *is_running.lock().unwrap() = false;
            }
        });
    }

    {
        let is_app_running = Arc::clone(&is_app_running);
        let is_running = Arc::clone(&is_running);
        let start = String::from("start");
        let stop = String::from("stop");
        let exit = String::from("exit");
        let help = String::from("help");

        thread::spawn(move || {
            let mut local_is_app_running = *is_app_running.lock().unwrap();

            while local_is_app_running {
                local_is_app_running = *is_app_running.lock().unwrap();

                let mut input: String = String::new();
                stdin()
                    .read_line(&mut input)
                    .expect("Did not enter correct string");

                input = input.trim().to_string();

                if start.eq(&input) {
                    println!("Resuming work");
                    *is_running.lock().unwrap() = true;
                    continue;
                }
                if stop.eq(&input) {
                    println!("Pausing work");
                    *is_running.lock().unwrap() = false;
                    continue;
                }
                if exit.eq(&input) {
                    println!("Exited the program");
                    *is_running.lock().unwrap() = false;
                    *is_app_running.lock().unwrap() = false;
                    continue;
                }
                if help.eq(&input) {
                    println!("start: start the app\nstop: pause the app\nexit: exit the app");
                    continue;
                }

                println!("Invalid command! Type help for a list of commands")
            }
        });
    }

    {
        thread::spawn(move || {
            let is_app_running = Arc::clone(&is_app_running);
            let is_running = Arc::clone(&is_running);

            let mut enigo = Enigo::new(&Settings::default()).unwrap();

            let mut time_passed = 0;
            while *is_app_running.lock().unwrap() {
                thread::sleep(one_second);
                while *is_running.lock().unwrap() {
                    thread::sleep(one_second);
                    time_passed += 1;

                    if time_passed >= KEY_PRESS_INTERVAL {
                        let _ = enigo.key(Key::DownArrow, Press);
                        let _ = enigo.key(Key::DownArrow, Release);
                        time_passed = 0;
                    }
                }
            }
        });
    }

    ui.run()?;

    Ok(())
}

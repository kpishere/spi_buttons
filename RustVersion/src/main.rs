use spibuttonlib::{SPIButtonController, SPIButtonState, SPIButton};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing SPI Buttons Controller on Beaglebone Black...");

    let mut controller = SPIButtonController::new(20, "/dev/spidev1.0", 800_000, 0)?;

    // Set default buttons
    for i in 0..20 {
        let btn = SPIButton::new(SPIButtonState::Off as u8 | SPIButtonState::OnChange as u8 | SPIButtonState::OnHold as u8 | SPIButtonState::Toggle as u8);
        controller.set_button(i, btn);
    }

    loop {
        let events = controller.loop_once()?;

        // The application logic
        for i in 0..events.len() {
            let mut b = events[i];
            println!("Button {}: State {:?}", b.id(), b.get_state());
            if b.is_hold_event() {
                match b.get_state() {
                    SPIButtonState::Off => b.set_state(SPIButtonState::On),
                    SPIButtonState::On => b.set_state(SPIButtonState::Flash1),
                    SPIButtonState::Flash1 => b.set_state(SPIButtonState::Flash2),
                    SPIButtonState::Flash2 => b.set_state(SPIButtonState::Off),
                    _ => {}
                }
                b.clear_hold_event();
                controller.set_button(b.id(), b);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}


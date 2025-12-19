use spidev::{Spidev, SpidevOptions, SpiModeFlags, SpidevTransfer};
use std::fs::File;
use std::ptr;
use std::os::fd::AsRawFd;
use libc;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum SPIButtonState {
    Off = 0x00,
    On = 0x01,
    Flash1 = 0x02,
    Flash2 = 0x03,
    LampOn = 0x04,
    Toggle = 0x08,
    PressedLag1 = 0x10,
    OnChange = 0x20,
    OnHold = 0x40,
    HoldEvent = 0x80,
}

impl SPIButtonState {
    fn from_u8(value: u8) -> Self {
        match value & 0x03 {
            0 => SPIButtonState::Off,
            1 => SPIButtonState::On,
            2 => SPIButtonState::Flash1,
            3 => SPIButtonState::Flash2,
            _ => SPIButtonState::Off,
        }
    }
}

#[derive(Clone, Copy)]
struct SPIButton {
    data: u8,
    scans_pressed: u32,
    id: u8,
}

impl SPIButton {
    fn new(state: SPIButtonState) -> Self {
        SPIButton {
            data: state as u8,
            scans_pressed: 0,
            id: 0,
        }
    }

    fn get_state(&self) -> SPIButtonState {
        SPIButtonState::from_u8(self.data)
    }

    fn set_state(&mut self, state: SPIButtonState) {
        self.data &= !(SPIButtonState::Off as u8 | SPIButtonState::On as u8 | SPIButtonState::Flash1 as u8 | SPIButtonState::Flash2 as u8);
        self.data |= state as u8;
    }

    fn is_lamp_on(&self) -> bool {
        (self.data & SPIButtonState::LampOn as u8) != 0
    }

    fn set_lamp(&mut self, on: bool) {
        if on {
            self.data |= SPIButtonState::LampOn as u8;
        } else {
            self.data &= !(SPIButtonState::LampOn as u8);
        }
    }

    fn do_toggle(&self) -> bool {
        (self.data & SPIButtonState::Toggle as u8) != 0
    }

    fn last_scan(&self) -> bool {
        (self.data & SPIButtonState::PressedLag1 as u8) != 0
    }

    fn set_last(&mut self, on: bool) {
        if on {
            self.data |= SPIButtonState::PressedLag1 as u8;
        } else {
            self.data &= !(SPIButtonState::PressedLag1 as u8);
        }
    }

    fn on_change(&self) -> bool {
        (self.data & SPIButtonState::OnChange as u8) != 0
    }

    fn on_hold(&self) -> bool {
        (self.data & SPIButtonState::OnHold as u8) != 0
    }

    fn is_hold_event(&self) -> bool {
        (self.data & SPIButtonState::HoldEvent as u8) != 0
    }

    fn set_hold_event(&mut self, on: bool) {
        if on {
            self.data |= SPIButtonState::HoldEvent as u8;
        } else {
            self.data &= !(SPIButtonState::HoldEvent as u8);
        }
    }

    fn toggle(&mut self) {
        match self.get_state() {
            SPIButtonState::Off => self.set_state(SPIButtonState::On),
            SPIButtonState::On | SPIButtonState::Flash1 | SPIButtonState::Flash2 => self.set_state(SPIButtonState::Off),
            _ => {}
        }
    }
}

type SPIButtonEvents = Vec<SPIButton>;

struct SPIButtonController {
    spi: Spidev,
    button_count: usize,
    buttons: Vec<SPIButton>,
    xmit_buf: Vec<u8>,
    scans: u32,
    setdata: *mut u32,
    clrdata: *mut u32,
    pin_bit: u32,
}

impl SPIButtonController {
    fn new(button_count: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spi = Spidev::open("/dev/spidev1.0")?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(1_000_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;

        let bytes = (button_count + 7) / 8;
        let xmit_buf = vec![0; bytes];
        let buttons = vec![SPIButton::new(SPIButtonState::Off); button_count];

        // LAMP_LATCH_PIN equivalent: P8_10 GPIO68
        let latch_pin = 68;
        let (setdata, clrdata, pin_bit) = Self::setup_gpio_mem(latch_pin)?;

        Ok(SPIButtonController {
            spi,
            button_count,
            buttons,
            xmit_buf,
            scans: 0,
            setdata,
            clrdata,
            pin_bit,
        })
    }
    fn setup_gpio_mem(pin: u32) -> Result<(*mut u32, *mut u32, u32), Box<dyn std::error::Error>> {
        let bank = pin / 32;
        let bit = pin % 32;
        let pin_bit = 1 << bit;
        let base_addr = match bank {
            0 => 0x44E07000,
            1 => 0x4804C000,
            2 => 0x481AC000,
            3 => 0x481AE000,
            _ => return Err("Invalid GPIO bank".into()),
        };

        let mem_file = File::open("/dev/mem")?;
        let mem_fd = mem_file.as_raw_fd();
        let gpio_base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                0x1000,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                mem_fd,
                base_addr as i64,
            ) as *mut u8
        };
        if gpio_base == libc::MAP_FAILED as *mut u8 {
            return Err("mmap failed".into());
        }

        let oe = (gpio_base as usize + 0x134) as *mut u32;
        let setdata = (gpio_base as usize + 0x194) as *mut u32;
        let clrdata = (gpio_base as usize + 0x190) as *mut u32;

        // Set direction to output
        unsafe {
            let current_oe = ptr::read_volatile(oe);
            ptr::write_volatile(oe, current_oe & !pin_bit);
        }

        Ok((setdata, clrdata, pin_bit))
    }

    fn set_gpio(&self, value: bool) {
        if value {
            unsafe { ptr::write_volatile(self.setdata, self.pin_bit); }
        } else {
            unsafe { ptr::write_volatile(self.clrdata, self.pin_bit); }
        }
    }
    fn transfer(&mut self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut rx_buf = vec![0u8; data.len()];
        let mut transfer = SpidevTransfer::read_write(data, rx_buf.as_mut_slice());
        self.spi.transfer(&mut transfer)?;
        Ok(rx_buf)
    }

    fn set_button(&mut self, pos: usize, mut button: SPIButton) {
        button.id = pos as u8;
        self.buttons[pos] = button;
    }

    fn get_button(&self, pos: usize) -> SPIButton {
        self.buttons[pos]
    }

    fn set_output_buffer(&mut self) {
        const SCANS_FLASH1: u32 = 3;
        const SCANS_FLASH2: u32 = 1;

        for b in 0..self.button_count {
            let mut spi_btn = self.get_button(b);
            match spi_btn.get_state() {
                SPIButtonState::Off => spi_btn.set_lamp(false),
                SPIButtonState::On => spi_btn.set_lamp(true),
                SPIButtonState::Flash1 => {
                    let flash = if self.scans % SCANS_FLASH1 == 0 { !spi_btn.is_lamp_on() } else { spi_btn.is_lamp_on() };
                    spi_btn.set_lamp(flash);
                }
                SPIButtonState::Flash2 => {
                    let flash = if self.scans % SCANS_FLASH2 == 0 { !spi_btn.is_lamp_on() } else { spi_btn.is_lamp_on() };
                    spi_btn.set_lamp(flash);
                }
                _ => {}
            }
            self.set_button(b, spi_btn);

            // For animation, lamp state is altered
            let spi_btn = self.get_button(b);
            let lamp_state = spi_btn.is_lamp_on();
            let byte_idx = b / 8;
            let bit_idx = b % 8;
            let btn_pressed = (self.xmit_buf[byte_idx] & (1 << bit_idx)) == 0; // High is un-pressed
            if (btn_pressed && !lamp_state) || (!btn_pressed && lamp_state) {
                self.xmit_buf[byte_idx] |= 1 << bit_idx; // set bit (light off)
            } else {
                self.xmit_buf[byte_idx] &= !(1 << bit_idx); // clear bit (light on)
            }
        }
    }

    fn get_input_buffer(&mut self, received: &[u8], events: &mut SPIButtonEvents) {
        const SCANS_ISHOLD: u32 = 10;

        for b in 0..self.button_count {
            let mut btn = self.get_button(b);
            let byte_idx = b / 8;
            let bit_idx = b % 8;
            let btn_pressed = (received[byte_idx] & (1 << bit_idx)) == 0; // High is un-pressed
            let is_hold = btn.scans_pressed > SCANS_ISHOLD;
            let is_down = btn_pressed && btn_pressed != btn.last_scan();
            let is_up = !btn_pressed && btn_pressed != btn.last_scan();

            // Update hold count
            btn.scans_pressed = if btn_pressed { btn.scans_pressed + 1 } else { 0 };

            if btn.on_change() && (is_down || is_up) {
                btn.set_hold_event(false);
                events.push(btn);
            }
            if btn.on_hold() && is_hold {
                btn.set_hold_event(true);
                events.push(btn);
            }

            if btn.do_toggle() && is_down {
                btn.toggle();
            }
            btn.set_last(btn_pressed);
            self.set_button(b, btn);
        }
    }

    fn loop_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut events = SPIButtonEvents::new();
        let xmit_data = self.xmit_buf.clone();
        self.set_gpio(false); // Latch low to read buttons
        let rx_buf = self.transfer(&xmit_data)?;
        self.get_input_buffer(&rx_buf, &mut events);
        self.set_gpio(true);  // Latch high to end read 
        self.set_output_buffer(); // Update for lights
        let xmit_data2 = self.xmit_buf.clone();
        let _ = self.transfer(&xmit_data2)?;
        for i in 0..events.len() {
            let b = events[i];
            println!("Button {}: State {:?}", b.id, b.get_state());
            if b.is_hold_event() {
                let mut btn = self.get_button(b.id as usize);
                match btn.get_state() {
                    SPIButtonState::Off => btn.set_state(SPIButtonState::On),
                    SPIButtonState::On => btn.set_state(SPIButtonState::Flash1),
                    SPIButtonState::Flash1 => btn.set_state(SPIButtonState::Flash2),
                    SPIButtonState::Flash2 => btn.set_state(SPIButtonState::Off),
                    _ => {}
                }
                btn.scans_pressed = 0;
                self.set_button(b.id as usize, btn);
            }
        }
        self.scans += 1;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing SPI Buttons Controller on Beaglebone Black...");

    let mut controller = SPIButtonController::new(20)?;

    // Set default buttons
    let default_state = SPIButtonState::Off as u8 | SPIButtonState::OnChange as u8 | SPIButtonState::OnHold as u8 | SPIButtonState::Toggle as u8;
    for i in 0..20 {
        let mut btn = SPIButton::new(SPIButtonState::Off);
        btn.data |= default_state;
        controller.set_button(i, btn);
    }

    loop {
        controller.loop_once()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_button_new() {
        let btn = SPIButton::new(SPIButtonState::On);
        assert_eq!(btn.get_state(), SPIButtonState::On);
        assert_eq!(btn.data, 0x01);
    }

    #[test]
    fn test_spi_button_set_state() {
        let mut btn = SPIButton::new(SPIButtonState::Off);
        btn.set_state(SPIButtonState::On);
        assert_eq!(btn.get_state(), SPIButtonState::On);
    }

    #[test]
    fn test_spi_button_set_lamp() {
        let mut btn = SPIButton::new(SPIButtonState::Off);
        btn.set_lamp(true);
        assert!(btn.is_lamp_on());
        btn.set_lamp(false);
        assert!(!btn.is_lamp_on());
    }

    #[test]
    fn test_spi_button_toggle() {
        let mut btn = SPIButton::new(SPIButtonState::Off);
        btn.toggle();
        assert_eq!(btn.get_state(), SPIButtonState::On);
        btn.toggle();
        assert_eq!(btn.get_state(), SPIButtonState::Off);
    }

    #[test]
    fn test_spi_button_state_from_u8() {
        assert_eq!(SPIButtonState::from_u8(0), SPIButtonState::Off);
        assert_eq!(SPIButtonState::from_u8(1), SPIButtonState::On);
        assert_eq!(SPIButtonState::from_u8(2), SPIButtonState::Flash1);
        assert_eq!(SPIButtonState::from_u8(3), SPIButtonState::Flash2);
        assert_eq!(SPIButtonState::from_u8(4), SPIButtonState::Off); // since & 0x03 == 0
    }
}

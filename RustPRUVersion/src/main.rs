use pru;
use std::fs;
use std::thread;
use std::time::Duration;

const PRU_DATA_BUFFER_SIZE: usize = 0x400;

#[repr(C)]
struct PruSpiContext {
    buffers: [u8; PRU_DATA_BUFFER_SIZE * 2],
    buffer: u32,
    length: u32,
    slave_max_transmission_length: u32,
}

#[derive(Clone, Copy, Debug)]
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

struct SPIButtonEvent {
    events: Vec<SPIButton>,
    count: usize,
}

impl SPIButtonEvent {
    fn new() -> Self {
        SPIButtonEvent {
            events: Vec::new(),
            count: 0,
        }
    }

    fn add(&mut self, button: SPIButton) {
        self.events.push(button);
        self.count += 1;
    }
}

struct SPIButtonController {
    pru: pru::Pru,
    context: *mut PruSpiContext,
    button_count: usize,
    buttons: Vec<SPIButton>,
    xmit_buf: Vec<u8>,
    scans: u32,
    latch_pin: u32,
}

impl SPIButtonController {
    fn new(button_count: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let pru = pru::Pru::new(0)?;
        pru.load_firmware("pru-spi-master.bin")?;
        pru.run();
        let context = pru.map_ram(0, std::mem::size_of::<PruSpiContext>()) as *mut PruSpiContext;

        let bytes = (button_count + 7) / 8;
        let xmit_buf = vec![0; bytes];
        let buttons = vec![SPIButton::new(SPIButtonState::Off); button_count];

        // LAMP_LATCH_PIN equivalent: P8_10 GPIO68
        let latch_pin = 68;
        Self::setup_gpio(latch_pin)?;

        Ok(SPIButtonController {
            pruss,
            context,
            button_count,
            buttons,
            xmit_buf,
            scans: 0,
            latch_pin,
        })
    }
    fn setup_gpio(pin: u32) -> Result<(), Box<dyn std::error::Error>> {
        fs::write("/sys/class/gpio/export", pin.to_string())?;
        fs::write(format!("/sys/class/gpio/gpio{}/direction", pin), "out")?;
        Ok(())
    }

    fn set_gpio(pin: u32, value: bool) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(format!("/sys/class/gpio/gpio{}/value", pin), if value { "1" } else { "0" })?;
        Ok(())
    }
    fn transfer(&mut self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let len = data.len();
        unsafe {
            (*self.context).length = len as u32;
            (*self.context).buffer = 0;
            std::ptr::copy_nonoverlapping(data.as_ptr(), (*self.context).buffers.as_mut_ptr(), len);
        }
        // Wait for PRU to complete
        while unsafe { (*self.context).length } != 0 {
            thread::sleep(Duration::from_micros(10));
        }
        let mut rx = vec![0; len];
        unsafe {
            std::ptr::copy_nonoverlapping((*self.context).buffers.as_ptr(), rx.as_mut_ptr(), len);
        }
        Ok(rx)
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

    fn get_input_buffer(&mut self, received: &[u8], event: &mut SPIButtonEvent) {
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
                event.add(btn);
            }
            if btn.on_hold() && is_hold {
                btn.set_hold_event(true);
                event.add(btn);
            }

            if btn.do_toggle() && is_down {
                btn.toggle();
            }
            btn.set_last(btn_pressed);
            self.set_button(b, btn);
        }
    }

    fn loop_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        rx_buf = self.transfer(&self.xmit_buf)?;
        self.get_input_buffer(&rx_buf, &mut event);
        // HIGH: Button parallel read & Lights serial set on
        Self::set_gpio(self.latch_pin, true)?;
        self.set_output_buffer(); // Update for lights
        let _ = self.transfer(&self.xmit_buf)?;
        // HIGH: Button parallel read & Lights serial set on
        Self::set_gpio(self.latch_pin, true)?;
        let mut transfer2 = SpidevTransfer::read_write(&self.xmit_buf, rx_buf.as_mut_slice());
        self.spi.transfer(&mut transfer2).unwrap();
        self.set_output_buffer(); // Update for lights
        if event.count > 0 {
            for i in 0..event.count {
                let b = event.events[i];
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
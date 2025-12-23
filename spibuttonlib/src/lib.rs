use spidev::{Spidev, SpidevOptions, SpiModeFlags, SpidevTransfer};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum SPIButtonState {
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
    pub fn from_u8(value: u8) -> Self {
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
pub struct SPIButton {
    data: u8,
    scans_pressed: u32,
    id: u8,
}

impl SPIButton {
    pub fn new(state: u8) -> Self {
        SPIButton {
            data: state as u8,
            scans_pressed: 0,
            id: 0,
        }
    }

    pub fn get_state(&self) -> SPIButtonState {
        SPIButtonState::from_u8(self.data & (SPIButtonState::Off as u8
            | SPIButtonState::On as u8
            | SPIButtonState::Flash1 as u8
            | SPIButtonState::Flash2 as u8)) 
    }

    pub fn set_state(&mut self, state: SPIButtonState) {
        self.data &= !(SPIButtonState::Off as u8 | SPIButtonState::On as u8 | SPIButtonState::Flash1 as u8 | SPIButtonState::Flash2 as u8);
        self.data |= state as u8;
    }

    pub fn id(&mut self) -> u8 {
        self.id
    }

    pub fn is_hold_event(&self) -> bool {
        (self.data & SPIButtonState::HoldEvent as u8) == SPIButtonState::HoldEvent as u8
    }

    pub fn clear_hold_event(&mut self) {
        self.scans_pressed = 0;
    }

    fn is_lamp_on(&self) -> bool {
        (self.data & SPIButtonState::LampOn as u8) == SPIButtonState::LampOn as u8
    }

    fn set_lamp(&mut self, on: bool) {
        if on {
            self.data |= SPIButtonState::LampOn as u8;
        } else {
            self.data &= !(SPIButtonState::LampOn as u8);
        }
    }

    fn do_toggle(&self) -> bool {
        (self.data & SPIButtonState::Toggle as u8) == SPIButtonState::Toggle as u8
    }

    fn last_scan(&self) -> bool {
        (self.data & SPIButtonState::PressedLag1 as u8) == SPIButtonState::PressedLag1 as u8
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

pub type SPIButtonEvents = Vec<SPIButton>;

pub struct SPIButtonController {
    spi: Spidev,
    button_count: usize,
    buttons: Vec<SPIButton>,
    xmit_buf: Vec<u8>,
    scans: u32,
}

impl SPIButtonController {
    pub fn new(button_count: usize, spidev: &str,  bus_speed: u32, mode: u8) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spi = Spidev::open(spidev)?;
        let modeval = match mode {
	    0 => SpiModeFlags::SPI_MODE_0,
	    1 => SpiModeFlags::SPI_MODE_1,
	    2 => SpiModeFlags::SPI_MODE_2,
	    3 => SpiModeFlags::SPI_MODE_2,
	    _ => panic!("Invalid SPI mode in config"),
	}; 
	let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(bus_speed)
            .mode(modeval)
            .build();
        spi.configure(&options)?;

        let bytes = (button_count + 7) / 8;
        let xmit_buf = vec![0; bytes];
        let buttons = vec![SPIButton::new(SPIButtonState::Off as u8); button_count];

        Ok(SPIButtonController {
            spi,
            button_count,
            buttons,
            xmit_buf,
            scans: 0,
        })
    }

    pub fn set_button(&mut self, pos: u8, mut button: SPIButton) {
        button.id = pos;
        self.buttons[pos as usize] = button;
    }

    pub fn get_button(&self, pos: usize) -> SPIButton {
        self.buttons[pos]
    }

    pub fn loop_once(&mut self) -> Result<SPIButtonEvents, Box<dyn std::error::Error>> {
        let mut events = SPIButtonEvents::new();
        self.set_output_buffer(); 
        let xmit_data = self.xmit_buf.clone();
        let rx_buf = self.transfer(&xmit_data)?;
        self.get_input_buffer(&rx_buf, &mut events);
        self.scans += 1;
        Ok(events)
    }

    fn reversebits(v:u8) -> u8{
        (v & 0b00000001) << 7 
        | (v & 0b00000010) << 5
        | (v & 0b00000100) << 3
        | (v & 0b00001000) << 1
        | (v & 0b00010000) >> 1
        | (v & 0b00100000) >> 3
        | (v & 0b01000000) >> 5 
        | (v & 0b10000000) >> 7
    }
    
    fn transfer(&mut self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut rx_buf  = vec![0u8; data.len()];

        // Bit reversing may/may-not be needed depending upon architectore of SPI master
        // Beaglebone -- need it, Atmel -- don't need it
        //
        let mut rx_rbuf = vec![0u8; data.len()];        
        let mut rdata = vec![0u8; data.len()];
        for i in 0..data.len() {
            rdata[i] = SPIButtonController::reversebits(data[i]);
        }

        let mut transfer = SpidevTransfer::read_write(rdata.as_slice(), rx_buf.as_mut_slice() );
        self.spi.transfer(&mut transfer)?;

        for i in 0..rx_buf.len() {
            rx_rbuf[i] = SPIButtonController::reversebits(rx_buf[i]);
        }
        Ok(rx_rbuf)
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
            // For animation, lamp state is altered
            let lamp_state = spi_btn.is_lamp_on();
            let byte_idx = b / 8;
            let bit_idx = b % 8;
            if lamp_state {
                self.xmit_buf[byte_idx] |= 1 << bit_idx; // set bit (light on)
            } else {
                self.xmit_buf[byte_idx] &= !(1 << bit_idx); // clear bit (light off)
            }
            self.set_button(b as u8, spi_btn);
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
            if btn_pressed { btn.scans_pressed = btn.scans_pressed + 1 } else { btn.scans_pressed = 0 };

            if btn.on_change() && ( is_down || is_up ) {
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
            self.set_button(b as u8, btn);
        }
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

#include "src/SPIButton.h"

#define SPIBUTTON_COUNT 20

////
// A demo program that tests the SPIButton class on an Arduino MEGA2560.
//
// Keypress and hold events are output to serial port with some state values shown on button.
// The hold event is used to toggle the state of the indicator on the button.
//
// In a real application, you'd want to trigger the scanning by a timer trigger event.
//
// Au: kpishere 2021
////

void mySPIButtonEventHandler(SPIButtonController &controller, SPIButtonEvent events) {
  for(int i =0; i < events.count; i++) {
    SPIButton b = events.events[i];
    SPIButtonState bs = b.getState();
    Serial.print(b.id);
    Serial.print(b.isHoldEvent() ? " HOLD " : " ");
    Serial.print(b.doToggle() ? " TGL " : " ");
    Serial.print(b.isLampOn() ? " LON " : " ");
    Serial.print(b.lastScan() ? " LAG_1 " : " LAG_0 ");
    Serial.print(b.onChange() ? " ONCHG " : " ");
    Serial.print(b.onHold()   ? " ONHLD " : " ");
    Serial.println(bs, BIN);

    if(b.isHoldEvent())
    {
      switch(b.getState()) {
        case SPIButtonStateE::SPIButtonState_Off:
          b.setState( SPIButtonStateE::SPIButtonState_On );
        break;
        case SPIButtonStateE::SPIButtonState_On:
          b.setState( SPIButtonStateE::SPIButtonState_Flash1 );
        break;
        case SPIButtonStateE::SPIButtonState_Flash1:
          b.setState( SPIButtonStateE::SPIButtonState_Flash2 );
        break;
        case SPIButtonStateE::SPIButtonState_Flash2:
          b.setState( SPIButtonStateE::SPIButtonState_Off );
        break;
      }
      b.scansPressed = 0; // reset hold button measure
      controller.setButton(b.id,b);
    }
  }
}

// Globals
SPIButtonController *btnCtrl;
uint8_t xmit[SPIBUTTON_BYTES(SPIBUTTON_COUNT)];

void setup() {
  SPIButton defaultButton = SPIButton((SPIButtonState)(SPIButtonStateE::SPIButtonState_Off | SPIButtonStateE::SPIButtonEventMask_OnChange | SPIButtonStateE::SPIButtonEventMask_OnHold));

  Serial.begin(115200);

  // 1 MHz clock, MSB first, mode 0
  btnCtrl = new SPIButtonController(SPISettings(1E6, LSBFIRST, SPI_MODE0), SPIBUTTON_COUNT, xmit, &mySPIButtonEventHandler);

  // Button config
  for(int i = 0; i < SPIBUTTON_COUNT; i++) btnCtrl->setButton(i,defaultButton);
}

void loop() {
  btnCtrl->loop();
  delay (100);
}

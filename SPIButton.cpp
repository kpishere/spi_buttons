#include "SPIButton.h"

SPIButtonController::SPIButtonController(SPISettings spiSettings, uint8_t buttonCount, uint8_t *xmitBuff, SPIButtonEventCall callback ) {
  SPI.begin();
  _spiSettings = spiSettings;
  _buttonCount = buttonCount;
  _xmitBuf = xmitBuff;
  _callback = callback;
  _scans = 0;
  _buttons = new SPIButton [_buttonCount];

  digitalWrite(SS,HIGH);

  // Lamp strobe latch outpt enable always
  pinMode(LAMP_LATCH_PIN, OUTPUT);
};
SPIButtonController::~SPIButtonController() {
  delete[] _buttons;
  SPI.end();
}
SPIButtonController::loop() {
  byte c;

  SPI.beginTransaction (_spiSettings);

  digitalWrite(LAMP_LATCH_PIN, LOW);

  // LOW: Button serial read & Lights serial set off
  digitalWrite(SS, LOW);
  SPI.transfer(_xmitBuf, SPIBUTTON_BYTES(_buttonCount));
  getInputBuffer();

  digitalWrite(LAMP_LATCH_PIN, HIGH);
  
  // High: Button parallel read & Lights serial set on
  digitalWrite(SS, HIGH);
  setOutputBuffer();
  SPI.transfer(_xmitBuf, SPIBUTTON_BYTES(_buttonCount));

  SPI.endTransaction();
  _scans++;
};

void SPIButtonController::setOutputBuffer() {
  boolean flash;

 // Copy light status to buffer
  for(int b = 0; b < _buttonCount; b++)  {
    SPIButton spiBtn = this->getButton(b);
    switch(spiBtn.getState())
    {
      case SPIButtonStateE::SPIButtonState_Off: spiBtn.setLamp(false); break;
      case SPIButtonStateE::SPIButtonState_On:  spiBtn.setLamp(true);  break;
      case SPIButtonStateE::SPIButtonState_Flash1:
        flash = spiBtn.isLampOn();
        // toggle state after N cycles
        if( _scans % SPIBUTTON_SCANS_FLASH1 == 0 ) flash = (flash ? false : true);
        spiBtn.setLamp(flash);
      break;
      case SPIButtonStateE::SPIButtonState_Flash2:
        flash = spiBtn.isLampOn();
        // toggle state after N cycles
        if( _scans % SPIBUTTON_SCANS_FLASH2 == 0 ) flash = (flash ? false : true);
        spiBtn.setLamp(flash);
      break;
    }
    this->setButton(b,spiBtn); // update for button lamp state

    // For animation, lamp state is altered
    boolean lampState = spiBtn.isLampOn();
    // buttons were just read and were about to update for output
    boolean btnPressed = SPIBUTTON_BIT(_xmitBuf,b) ? false : true; // High is un-pressed
    if( (btnPressed ? !lampState : lampState) ) SPIBUTTON_BIT_SET(_xmitBuf,b);// set bit (light off)
    else SPIBUTTON_BIT_CLR(_xmitBuf,b); // clear bit (light on)
  }
};

void SPIButtonController::getInputBuffer() {
  SPIButtonEvent newEvent;
  newEvent.clear();

  // copy button status from buffer
  for(int b = 0; b < _buttonCount; b++)  {
    SPIButton btn = this->getButton(b);
    boolean btnPressed = SPIBUTTON_BIT(_xmitBuf,b) ? false : true; // High is un-pressed
    boolean isHold = (btn.scansPressed > SPIBUTTOM_SCANS_ISHOLD);
    boolean isDown = false, isUp = false;

    // Update hold count
    btn.scansPressed = (btnPressed ? btn.scansPressed + 1 : 0);

    isDown = (btnPressed && btnPressed != btn.lastScan() );
    isUp = (!btnPressed && btnPressed != btn.lastScan() );

    if(btn.onChange() && (isDown || isUp) ) {
      btn.setHoldEvent(false);
      newEvent.add(btn);
    }
    if(btn.onHold() && isHold ) {
      btn.setHoldEvent(true);
      newEvent.add(btn);
    }

    if( btn.doToggle() && isDown) btn.toggle();
    btn.setLast(btnPressed);
    this->setButton(b,btn);
  }
  // make callback
  if(newEvent.count > 0) this->_callback(*this,newEvent);
}

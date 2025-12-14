#pragma once

#include <SPI.h>

////
// A shift register controller that uses SPI hardware bus
//
//  Supports buttons that have a corresponding light indicator.
//
//  Tested with buttons connected to MC14021B parallel to serial converters
//  and lights connected to MC14094B serial to parallel converters.
//
//  AU: kpishere 2021
////

#define LAMP_LATCH_PIN 49

// maximum press events in one callback (how many buttons pressed at once)
#define SPIBUTTON_EVENTMAX 4

// Bit utillity functions
#define SPIBUTTON_BYTES(COUNT) ((unsigned int)(COUNT)/8)+((COUNT) % 8 > 0 ? 1 : 0)
#define SPIBUTTON_BIT(byteBuf,pos)  (byteBuf[pos/8] & (1 << (pos % 8)))
#define SPIBUTTON_BIT_SET(byteBuf,pos) byteBuf[b/8]|= (1 << (pos % 8))
#define SPIBUTTON_BIT_CLR(byteBuf,pos) byteBuf[b/8]&= ~(1 << (pos % 8))

// Scans until lamp illumination is toggled -- these numbers may vary depending upon processor
// Following were suitable for an Atmel at 16Mhz
//
#define SPIBUTTON_SCANS_FLASH1 3
#define SPIBUTTON_SCANS_FLASH2 1
// Consecutive button scans until press is considered a hold
#define SPIBUTTOM_SCANS_ISHOLD 10
//
//////

typedef enum SPIButtonStateE:uint8_t  {
// b0000 00XX -- Button can have these states of light status
  SPIButtonState_Off      = 0x00
, SPIButtonState_On       = 0x01
, SPIButtonState_Flash1   = 0x02
, SPIButtonState_Flash2   = 0x03
// b0000 0X00 - Lamp On/Off is independant of state
, SPIButtonLamp_On = 0x04
// b0000 X000 - Switch between On and off state in one cycle when pressed each time
, SPIButtonBehave_Toggle  = 0x08
// b000X 0000 - Pressed status of last scan
, SPIButtonPressed_Lag1   = 0x10
// b00X0 0000 - Events triggered on a press event
, SPIButtonEventMask_OnChange= 0x20
// b0X00 0000 - Events triggere on a hold event (repeats when held)
, SPIButtonEventMask_OnHold  = 0x40
// bX000 0000 - Indicates if event is a Hold (repeat) event for the same press
, SPIButtonEvent_IsHold      = 0x80
} SPIButtonState;

typedef struct SPIButtonS {
  SPIButtonState data;
  unsigned int scansPressed;
  uint8_t id;

  SPIButtonS() {}
  SPIButtonS(SPIButtonState state)
  {
    data = state;
  }

  // get state values
  inline SPIButtonState getState() { return data & (SPIButtonStateE::SPIButtonState_Off | SPIButtonStateE::SPIButtonState_On | SPIButtonStateE::SPIButtonState_Flash1 | SPIButtonStateE::SPIButtonState_Flash2); }
  inline boolean isLampOn() { return ((data & SPIButtonStateE::SPIButtonLamp_On) == SPIButtonStateE::SPIButtonLamp_On); }
  inline boolean doToggle() { return (data & SPIButtonStateE::SPIButtonBehave_Toggle) == SPIButtonStateE::SPIButtonBehave_Toggle; }
  inline boolean lastScan() { return (data & SPIButtonStateE::SPIButtonPressed_Lag1) == SPIButtonStateE::SPIButtonPressed_Lag1; }
  inline boolean onChange() {return (data & SPIButtonStateE::SPIButtonEventMask_OnChange) != 0x00; }
  inline boolean onHold() {return (data & SPIButtonStateE::SPIButtonEventMask_OnHold    ) != 0x00; }
  inline boolean isHoldEvent() {return (data & SPIButtonStateE::SPIButtonEvent_IsHold) == SPIButtonStateE::SPIButtonEvent_IsHold; }

  // set/adjust state values
  inline void setState(SPIButtonState oneOfStates) {
    // clear all
    data &= ~(SPIButtonStateE::SPIButtonState_Off | SPIButtonStateE::SPIButtonState_On | SPIButtonStateE::SPIButtonState_Flash1 | SPIButtonStateE::SPIButtonState_Flash2);
    // set new
    data |= oneOfStates;
  }
  inline void toggle() {
    switch( getState() ) {
      case SPIButtonStateE::SPIButtonState_Off:
        setState( SPIButtonStateE::SPIButtonState_On );
      break;
      case SPIButtonStateE::SPIButtonState_On:
      case SPIButtonStateE::SPIButtonState_Flash1:
      case SPIButtonStateE::SPIButtonState_Flash2:
        setState( SPIButtonStateE::SPIButtonState_Off );
      break;
    }
  }
  inline void setLamp(boolean on) { if(on) data |= SPIButtonStateE::SPIButtonLamp_On; else data &= ~SPIButtonStateE::SPIButtonLamp_On; };
  inline void setLast(boolean on) { if(on) data |= SPIButtonStateE::SPIButtonPressed_Lag1; else data &= ~SPIButtonStateE::SPIButtonPressed_Lag1; };
  inline void setHoldEvent(boolean on) { if(on) data |= SPIButtonStateE::SPIButtonEvent_IsHold; else data &= ~SPIButtonStateE::SPIButtonEvent_IsHold; };
} SPIButton;

typedef struct SPIButtonEventS {
  SPIButton events[SPIBUTTON_EVENTMAX];
  unsigned int count;
  inline void clear() { count = 0; };
  inline void add(SPIButton button) { events[count++] = button; };
} SPIButtonEvent;

class SPIButtonController;

typedef void (*SPIButtonEventCall)(SPIButtonController &controller, SPIButtonEvent events);

class SPIButtonController {
  public:
    SPIButtonController(SPISettings spiSettings, uint8_t buttonCount, uint8_t *xmitBuff, SPIButtonEventCall callback );
    ~SPIButtonController();

    inline void setButton(uint8_t pos, SPIButton button) {
        button.id = pos;
        _buttons[pos] = button;
    }
    inline SPIButton getButton(uint8_t pos) {
        return _buttons[pos];
    }
    loop();
  private:
    void setOutputBuffer();
    void getInputBuffer();

    SPISettings _spiSettings;
    uint8_t _readSelectPin;
    uint8_t _buttonCount;
    SPIButton *_buttons;
    uint8_t *_xmitBuf;
    SPIButtonEventCall _callback;
    unsigned int _scans;
};

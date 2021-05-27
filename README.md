# SPI Buttons

A little weekender project to make a utility class for controlling some buttons with lights because well, there is just a little too much touch screen in the world today and such physical interfaces are becoming a little bit special -- and appreciated.

The driver and test rig for this project is what you see below.  A button panel from a salvaged video broadcast control unit of some sort (do you recognize this for the equipment from which it comes?).

The buttons have clear-cap pop-offs where you can put laser/jet printed text on mylar film under them and have backlit lamps.

The lamps in this case consume quite a bit of power (64mA each which totals about 1A when all are lit).

Feature wise, the class provides callback to your code for press down, press up, and hold events.  There is option to have class do the toggle on/off if that is all you need and get the callback on that event.  There are four button states : On, Off, Flash 1, and Flash 2.  Flash 2 is faster flashing than Flash 1.  The buttons are animated for positive feedback - you press them and the light inverts from off/on and visa versa.

[Don't ask me why that op-amp is in the upper left corner of board, I don't know.  It isn't used.  I'd noted that it's output is linked to a driver transistor for about 300mA capacity output. Pins to this circuit are independant, on the blue connector but are not connected.]

|Face|Board|
|----|-----|
|![IMG_3104](https://user-images.githubusercontent.com/25204173/119839212-01f25380-bed2-11eb-9abd-892c1008feb9.jpg)|![IMG_3102](https://user-images.githubusercontent.com/25204173/119839379-1fbfb880-bed2-11eb-9607-0ee061cb1d42.jpg)|

# Rattus
A modal Mouse Key meant to entirely replace the mouse Currently tested on Linux may work on Windows

*Keys:*

5: click

7: up-left

8: up

9: up-right

4: left

6: right

1: down-left

2: down

3: down-right

*Modes:*

0/Insert: toggle fast or slow movement

./Del: toggle right or left click

Rattus is under active development.

TODO: 

1. block numpad keycodes; currently this types into selection *done
2. create Rattus.service or other integration
3. add more modes: 
   
   -click and hold mode *done
   
   -record macro for repeating movements and clicks (also typing?) "Robot Mode"
      press enter on numpad then any of the numbers for a pre recorded operation at cursor position
      (9 "save slots")
   
   -enter numbers like normal keypad (arrows for navigation is num lock toggle this would be seperate) by holding shift
   
   -scroll wheel mode (1,2,3 are slow medium and fast down scroll respectively etc.) with the * button
   
   -history mode move to last click positions in buffer (up to n number of click positions) by tapping + to go back 
    and - forward up to cursor position when search began


Currently tested with command:
cargo build --release; sudo target/release/Rattus 100 10 100

where number arguments are: fast speed, slow speed, and output frequency

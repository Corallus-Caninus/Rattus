# Rattus
A modal Mouse Key meant to entirely replace the mouse Currently tested on Linux (X11 only) may work on Windows

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
   
   -middle click
   
   -record macro for repeating movements and clicks (also typing?) "Robot Mode"
      press enter on numpad then any of the numbers for a pre recorded operation at cursor position
      (9 "save slots")
   
   -rewrite numlock as arrow and number modes and use stateful representation since NKRO can cause numlock 
    keycode packet headers to arrive out of order.
   
   -scroll wheel mode (1,2,3 are slow medium and fast down scroll respectively etc.) with the * button
   
   -history mode move to last click positions in buffer (up to n number of click positions) by tapping + to go back 
    and - forward up to cursor position when search began
    
   -prediction mode toggle mode then move. when a TF model determines with confidence threshold where the next click or hover will occur it
    moves to that position interrupting user input. model takes input vector: ffmpeg or X11 equivalent of screen and mouse data outputs 
    probability and position (cannot click can only teleport then exit predict mode). If user doesnt click or hover within a bounding box 
    of teleport destination the prediction was incorrect else correct use reinforcement learning to train online with data from all movement 
    modes.
    


Currently tested with command:
cargo build --release; sudo target/release/Rattus 5000 500

where number arguments are: fast speed and slow speed

# Rattus
A modal Mouse Key meant to entirely replace the mouse Currently tested on Linux may work on Windows

*Keys:*

5: click

7: diagonal-up

8: up

9: diagonal-up

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

1. block numpad keycodes; currently this types into selection
2. create Rattus.service or other integration
3. add more modes: 
   -click and hold mode
   -moving to a quadrant/octant of the screen
   -record macro for repeating movements and clicks (also typing?)


Currently tested with command:
cargo build --release; sudo target/release/Rattus 100 10 100

where number arguments are: fast speed, slow speed, and output frequency

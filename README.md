# habit
 high altitude balloon interactive tester

the balloon and payload are physicalized. as parameters change, the view shows it in realtime. on initialization, the payload is tethered to the ground.

1. choose balloon parameters.
    - amount of gas inside balloon initially
    - type of gas inside balloon initially
    - size of payload
    - mass of payload
    - shape of payload
    - initial volume of balloon
    - emissivity of balloon
    - thickness of balloon material
    - type of balloon material
    - diameter of vent (circular valve like car intake)
    - speed of vent actuation
2. launch the balloon (cut the tether)
3. press space to open the vent
4. press backspace to cut the balloon<->payload tether

goal: maximize flight time.

game loop: flappy bird meets polybridge.
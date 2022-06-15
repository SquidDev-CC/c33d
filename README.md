# c33d
The laziest 3D renderer for CC.

I've been wanting to create a "window" with ComputerCraft's monitors for a
while, [rendering a 3D scene from the perspective of the player][demo]. However,
CC isn't really fast enough to do it well, and so we need to cheat.

c33d is a rust webserver which accepts world data and a player's relative
position. For each pixel on the monitor, we get the look vector from the player
to that pixel, and then cast a ray from the pixel through the provided world.
The resulting ray traces are packed into a buffer, sent back to the computer,
and drawn on a monitor.

This is incredibly incomplete. It only works on north facing monitors (i.e.
left corner is +X, right corner is -X), and works with a very limited number of
blocks (stone, grass, dirt, water).

[demo]: https://twitter.com/CuriousCalamari/status/1515785771009069061

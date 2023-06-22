# Cherry Game

This is a simple game build as a fullstack webapp, written in rust. The frontend is using yew as a framework and is built with trunk. As backend framework actix_web is used and the 
spa (single page application) crate from actix_lab to serve both the frontend and the backend in the final code. Goal of this project was for me to learn how 
to write a web applications in rust. 

The game could be described as "dynamic go". It is played between two connected players, who are given the colors red and blue. Every two seconds a player is allowed to claim a tile, alternating between players. 
After two seconds pass, the playing field "evolves", meaning every claimed tile  automatically claims all surrounding tiles. Winner of the game is the player who claimed more tiles. 

communication between front and back is done via http, the game state is kept in the backend and in session cookies.  

To build the project move to the frontend folder and run "trunk build". Then run the application from the backend folder with "cargo run". Running both frontend
and backend with one command is accomplished with "spa" from "actix_lab". 

[trunk]: https://github.com/thedodd/trunk
[yew]: https://github.com/yewstack/yew
[actix_web]: https://github.com/actix/actix-web


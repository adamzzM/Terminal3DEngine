
3d engine fully implemented in rust (almost) no external libraries except for some minor windows api things for getting the terminal size and input


### spinning skull
<p align="center">

  <img src="imgs/skull_spinning.gif" width="500">
</p>


Basic ahh 3d engine

    I still need to seperate the renderer struct into an engine struct and a renderer struct and I want to abstract some things further then I can use this project to build some cooler stuff like physics engines and games

this was a pretty fun project to work on , I got to understand a lot about engines like why they use 4d Matrices and some optimizations 
Also I was forced to learn some optimization and what my code was actually doing under the hood like when I should move , when I should clone , when to use references and when I should pass pointers. And the basic linear algebra was really nice 

3Blue1Browns series is a life saver

This is probably one of the first times actually switching from a quadratic time complexity solution to instantenous was actually noticable 


there lies a problem rn when it comes to implementing multi threading, I have two main ways of doing it rn 
either I can split the screen into n number of screens then compute each segement seperatly and have seperate z-buffers or I could split the array of triangles and compute them at the same time and then I'd have to atomize z-buffer to avoid problems with that 


## Running it

You need to have [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed 

clone the project `git clone https://github.com/adamzzM/Terminal3DEngine`

*important* before running you need to have an object file and have it linked in the main.rs currenly the preview is done with skull.obj (i dont remember where I got it from)

and then run `cargo run` inside of the folder

*A CPU-based, multithreaded software rasterizer
written from scratch in Rust.*


### spinning skull
<p align="center">

  <img src="imgs/skull_spinning.gif" width="500">
</p>

## Running it

You need to have [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed 

clone the project `git clone https://github.com/adamzzM/Terminal3DEngine`

*important* before running you need to have an object file and have it linked in the main.rs currenly the preview is done with skull.obj (i dont remember where I got it from)

and then run `cargo run` inside of the folder

## LICENSE 

its under the gnu license which basically means do whatever but it has to be open source too If you want to use it in a closed source application just dm me and I'll let you I just picked this because I genuinely have no idea 

## Features & How it works 

As of right now the Engine can load .obj files and .mtl files for objects and colors. Then for each frame it will calculate a Matrix from the transforms properties ( Position , Scale , Rotation ) then it will multiply each triangle in the mesh by this Matrix then rasterize the triangles (converting triangles into pixels) and then prints the buffer. 

Currently the rasterization of the triangles and the multiplication of them by the matrix is multi threaded on the cpu with no plans of offloading them to the gpu

## Future plans 

currently the state is still rather messy , I still plan to abstract everything a bit more so it be easier to build on top of and potentially build simulations or games over. But for now if you want to implement any sort of logic you would modify the Transforms given to you by the world struct. 

### random notes 

this project has taught me a lot in rust and in optimizing code to be faster , and some linear algebra (god bless 3blue1browns series)

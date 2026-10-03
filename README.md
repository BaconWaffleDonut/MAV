Simple game engine made using Rust and Vulkan. Currently supports both movement and rotation of the camera using fairly standard WASD\LCTRL\Space keybindings along with the mouse for rotation. Left shift increases speed tenfold when held. 

LOD's currently take effect too close to the camera, can be disabled by setting mip levels to 1 in engine_functions.rs 

Rendering object can be changed by manually editiing code in engine_functions.rs to the other picture and corresponding object files, may not work properly for every file given. 

Does require ./src/resources/ folder structure for both object and image files otherwise it will throw an error as those paths are hardcoded. 

ESC key also locks/unlocks the cursor to the window so it can be moved off screen or prevented from moving off the screen. Similar to how games will prevent the cursor from leaving the screen when in game but will allow the cursor to leave the window when like the pause menu is opened.

# mackerel 🐟
<sup><sub>mackerel is the translation of "maquereau" in french whichs sounds like "macro" :p</sub></sup>  
Light but powerful macro/hotkey software written in Rust

## Example
Define macros with a script like:
```
Mouse1 {
  press MouseLeft
  move to 1100, 640
  press MouseLeft
  move to start
}
```
And your script will be executed whenever you press Mouse button 1!  
*(an additionnal mouse button, usually on the left side)* 

## How to use
1. Either compile the program yourself (https://www.rust-lang.org/tools/install) or grab a release that matches your OS
1. Make sure 'listen.exe' is in the same folder as 'offset.toml' and 'macros.txt'
1. Tweak 'offset.toml' to match your screen config (see below)
1. Write your macro scripts in 'macros.txt'
1. Run 'listen.exe' and enjoy!  
*(preferably from an existing terminal instance so you can read the error messages if there's one)*

## Tweaking offset.toml
offset.toml shouldn't be needed at all, but unfortunately the library I use to simulate inputs (rdev) is bugged when multiple screens are involved.  
offset.toml contains an offset (x, y) and a scale (s) value to make the program work with your screen configuration.

The best method to configure it properly is to add the following script to 'macros.txt':
```
Mouse1 {
    move to start
}
```
If offset.toml is properly configured this script should do nothing at all, otherwise your mouse cursor will move and you will need to tweak the offset values until it doesn't.

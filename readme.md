# nostd web browser for the T-Deck


run with

```shell
. ~/export-esp.sh
SSID=wifi PASSWORD=wifi_password cargo run
```



next to test

[x] toggle button really works. state changes.
[x] toggle group really works. state changes.
[x] form layout properly lays out the form items
[x] touch debugger really draws at the touch surface
[x] add unit tests for these using a stub ctx and theme implementation.
[x] font button creates and opens a new menu view which then hides when the item is selected.
[ ] changing the theme and font really repaints the screen with new colors and font
[ ] be able to scroll and tab through links
[ ] draw text needs to support bold and underline and bg and fg color

* move the embedded specific code to a new sub crate
* make a simulator sub crate for testing the GUI
* The gui code stays in the main repo, but uses abstract types where possible so we can switch between embedded and desktop. 
* GUI operates on NetCommand channel to interact with the embedded network vs a std network.
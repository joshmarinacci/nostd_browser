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
* changing the theme really repaints the screen with new colors
* font button creates and opens a new menu view which then hides when the item is selected.
* touch debugger really draws at the touch surface
* add unit tests for these using a stub ctx and theme implementation.
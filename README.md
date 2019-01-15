# Chip8 interpreter in Rust

```bash
$ cargo run --release -- --help
$ cargo run --release games/PONG
```
## Virtual Key mappings

The original CHIP-8 had 16 virtual keys had the layout on the left, which has
been mapped (by default) to the keyboard layout on the right:

```
 1 2 3 C                                   1 2 3 4
 4 5 6 D    This is emulated with these    Q W E R
 7 8 9 E    keyboard keys -->              A S D F
 A 0 B F                                   Z X C V
```

To play PONG use <kbd>Q</kbd> and <kbd>1</kbd> to move the bar on the left up
and down and <kbd>4</kbd> and <kbd>R</kbd> for bar on the right.

## Notes

The flickering is caused by how the interpreter draws sprites onto the screen.
In particular, it performs a bitwise xor between the vram and the sprite to
draw. This ends up in flickering when drawing a sprite over an existing sprite
(`1 ^ 1 = 0`).

## Resources

- https://en.wikipedia.org/wiki/CHIP-8
- http://mattmik.com/files/chip8/mastering/chip8.html
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

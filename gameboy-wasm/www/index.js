import { Gameboy } from "./gameboy";

// Add a drag and drop mabye -- DataTransfer object.

const rom_input = document.getElementById("rom-input");
const gb = new Gameboy();

const canvas = document.getElementById("display");
const ctx = canvas.getContext("2d");

rom_input.addEventListener("change", () => {

    const file = rom_input.files[0];

    file.arrayBuffer().then((buffer) => {
        const buf = new Uint8Array(buffer);
        console.log(buffer.length);
        gb.boot(buf);
    });

}, false);

function resize_canvas(scale) {
    canvas.width = 160 * 2;
    canvas.height = 144 * 2;
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
}

resize_canvas(2);

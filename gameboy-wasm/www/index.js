import * as wasm from "gameboy-wasm";
// Add a drag and drop mabye -- DataTransfer object.

const testH1 = document.getElementById("test");
const rom_input = document.getElementById("rom-input");
rom_input.addEventListener("change", () => {
    const file = rom_input.files[0];
    testH1.textContent = file.name;
});

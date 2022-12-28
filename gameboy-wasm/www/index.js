import { Emulator, ram_size } from "gameboy-wasm";
import { memory } from "gameboy-wasm/gameboy_wasm_bg";

const rom = require("./pocket.gb");
const WIDTH = 160;
const HEIGHT = 144;
var scale = 4;
var timeout_id = null;

// Add a drag and drop mabye -- DataTransfer object.
const canvas = document.getElementById("display");
const ctx = canvas.getContext("2d");

ctx.font = 'bolder 25px Lucida Console';
ctx.textAlign = "center";
ctx.fillText("Choose a file\n to load as a ROM...", canvas.width/2, canvas.height/2);


const saveable_cartridges = [0x03, 0x06, 0x09, 0x0D, 0x0F, 0x10, 0x13, 0x1B, 0x1E, 0x22]
var saveable = false;

class Gameboy {
    
    boot(file, save) {

        clear_timeout();
        this.name = file.name.slice(0, -3);
        this.saveable = false;
        file.arrayBuffer().then((buffer) => {
            
            const buf = new Uint8Array(buffer);
            this.ram_size = ram_size(buf[0x0149]);

            if (save) {
                
                if (!saveable_cartridges.includes(buf[0x0147])) {
                    console.log("cartridge not of saveable type.")
                    this.gb = Emulator.new(buf, null);
                
                } else {
                    save.arrayBuffer().then((save_buffer) => {
                        const save_buf = new Uint8Array(save_buffer);
                        this.gb = Emulator.new(buf, save_buf);
                        saveable = true;
                    });

                    this.saveable = true;
                }
            } else {

                this.gb = Emulator.new(buf, null);
            }
            
            this.boot_key_listener();
            this.image_data = ctx.createImageData(WIDTH * scale, HEIGHT * scale);
            this.run();
        });
    }
    
    run() {
        timeout_id = setTimeout(this.run.bind(this), 1000 / 60);
        
        this.gb.tick();        
        const display_ptr = this.gb.pixels_ptr(scale);
        const buf = new Uint8ClampedArray(
            memory.buffer,
            display_ptr,
            WIDTH * HEIGHT * 16 * 4,
            );
        this.image_data.data.set(buf);
        
        ctx.putImageData(this.image_data, 0, 0, 0, 0, WIDTH * scale, HEIGHT * scale);
    }

    // save() {
    //     if (!this.saveable) {
    //         alert("Cartridge does not support saves.");
    //         return;
    //     }
    //     console.log("saving");
    //     const save_ptr = this.gb.get_save_data();
    //     const save_buf = new Uint8Array(
    //         memory.buffer,
    //         save_ptr,
    //         this.ram_size,
    //     );
    //     const file = new File(save_buf, `${this.name}.sav`);
    //     download_file(file);
    // }

    boot_key_listener() {
        window.addEventListener("keydown", (event) => {
            switch (event.key) {
                case "ArrowDown": {
                    this.gb.key_press(0);
                    break;
                }
                case "ArrowUp": {
                    this.gb.key_press(1);
                    break;
                }
                case "ArrowLeft": {
                    this.gb.key_press(2);
                    break;
                }
                case "ArrowRight": {
                    this.gb.key_press(3);
                    break;
                }
                case "z": {
                    this.gb.key_press(4);
                    break;
                }
                case "x": {
                    this.gb.key_press(5);
                    break;
                }
                case " ": {
                    this.gb.key_press(6);
                    break;
                }
                case "Enter": {
                    this.gb.key_press(7);
                    break;
                }      
            }
        });
        
        window.addEventListener("keyup", (event) => {
            switch (event.key) {
                case "ArrowDown": {
                    this.gb.key_release(0);
                    break;
                }
                case "ArrowUp": {
                    this.gb.key_release(1);
                    break;
                }
                case "ArrowLeft": {
                    this.gb.key_release(2);
                    break;
                }
                case "ArrowRight": {
                    this.gb.key_release(3);
                    break;
                }
                case "z": {
                    this.gb.key_release(4);
                    break;
                }
                case "x": {
                    this.gb.key_release(5);
                    break;
                }
                case " ": {
                    this.gb.key_release(6);
                    break;
                }
                case "Enter": {
                    this.gb.key_release(7);
                    break;
                }      
            }
        });
    }
}

const rom_input = document.getElementById("rom-input");
const gb = new Gameboy();

rom_input.addEventListener("change", () => {

    const files = rom_input.files;
    if (files === null || files === undefined) {
        console.log("none selected");
        return;
    }

    // Single file.
    if (files.length === 1) {
        const file = files[0];

        if (file.name.slice(-3, file.name.length) != ".gb" ) {
            console.log("not a gb file");
            return;
        }
        console.log("single file boot");
        gb.boot(file, null);

    // Multi file.
    } else {
        const rom = first_rom(files);
        const save = first_save(files);

        if (rom === null || rom === undefined) {
            console.log("need to supply  a rom");
            return;
        }

        if (save === null || save === undefined) {
            gb.boot(rom, null);
        } else {
            gb.boot(rom, save);
        }
    }
}, false);

var keys = {};
window.addEventListener("keydown",
    function(e){
        keys[e.keyCode] = true;
        switch(e.keyCode){
            case 37: case 39: case 38:  case 40: // Arrow keys
            case 32: e.preventDefault(); break; // Space
            default: break; // do not block other keys
        }
    },
false);
window.addEventListener("keyup",
    function(e){
        keys[e.keyCode] = false;
    },
false);

// const file_download = document.getElementById("file-button");
// file_download.addEventListener("click", () => {
//     if (gb === null || gb === undefined) {
//         return;
//     }
//     gb.save();
// })

function first_rom(file_list) {
    for (let i=0; i<file_list.length; i++) {
        const n = file_list.item(i).name;
        if (n.slice(-3, n.length) === ".gb") {
            return file_list.item(i);
        }
    }
    return null;
}

function first_save(file_list) {
    for (let i=0; i<file_list.length; i++) {
        const n = file_list.item(i).name;
        if (n.slice(-4, n.length) === ".sav") {
            return file_list.item(i);
        }
    }
    return null;
}

function clear_timeout() {
    if (timeout_id) {
        clearTimeout(timeout_id);
    }
    saveable = false;
}

// function download_file(file) {
//     const link = document.createElement("a");
//     link.style.display = "none";
//     link.href = URL.createObjectURL(file);
//     link.download = file.name;

//     document.body.appendChild(link);
//     link.click();
//     setTimeout(() => {
//         URL.revokeObjectURL(link.href);
//         link.parentNode.removeChild(link);
//     }, 0);
// }
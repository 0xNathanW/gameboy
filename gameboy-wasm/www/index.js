import { Emulator } from "gameboy-wasm";
import { memory } from "gameboy-wasm/gameboy_wasm_bg";

// Add a drag and drop mabye -- DataTransfer object.
const canvas = document.getElementById("display");
const ctx = canvas.getContext("2d");

const WIDTH = 160;
const HEIGHT = 144;
var scale = 4;



class Gameboy {
    
    boot(rom_data) {
        this.gb = Emulator.new(rom_data);
        
        this.boot_key_listener();
        this.image_data = ctx.createImageData(WIDTH * scale, HEIGHT * scale);
        
        this.run();
    }
    
    run() {
        setTimeout(this.run.bind(this), 1000 / 60);
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

const gb = new Gameboy();
const rom_input = document.getElementById("rom-input");

rom_input.addEventListener("change", () => {

    const file = rom_input.files[0];

    file.arrayBuffer().then((buffer) => {
        const buf = new Uint8Array(buffer);
        console.log(buffer.length);
        gb.boot(buf);
    });

}, false);
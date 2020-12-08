import './styles.scss';

// TODO: typecheck this
declare var wasm_bindgen: any;
const { run_emulator } = wasm_bindgen;

function initScreen() {
    const screen = document.getElementById('screen') as HTMLCanvasElement;
    screen.width = screen.clientWidth;
    screen.height = screen.clientHeight;
}

async function romChanged() {
    const rom = await new Promise((resolve, reject) => {
        const reader = new FileReader();

        reader.onload = function() {
            if (typeof this.result === 'string') {
                throw new Error('expected an ArrayBuffer');
            }

            resolve(new Uint8Array(this.result));
        };

        reader.onerror = reject;

        reader.readAsArrayBuffer(this.files[0]);
    });

    run_emulator(rom);
}

async function main() {
    await wasm_bindgen('./feo-boy/feo-boy_bg.wasm');

    initScreen();

    const romInput = document.getElementById('rom');
    romInput.addEventListener('change', romChanged, false)

    run_emulator();
}

main();

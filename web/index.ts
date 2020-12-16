import { run_emulator } from './pkg';

function initScreen() {
    const screen = document.getElementById('screen') as HTMLCanvasElement;
    screen.width = screen.clientWidth;
    screen.height = screen.clientHeight;
}

async function romChanged(this: HTMLInputElement) {
    const rom: Uint8Array = await new Promise((resolve, reject) => {
        const reader = new FileReader();

        reader.onload = function() {
            if (typeof this.result === 'string') {
                throw new Error('expected an ArrayBuffer');
            }

            resolve(new Uint8Array(this.result!));
        };

        reader.onerror = reject;

        reader.readAsArrayBuffer(this.files![0]);
    });

    run_emulator(rom);
}

async function main() {
    initScreen();

    const romInput = document.getElementById('rom') as HTMLInputElement;
    romInput.addEventListener('change', romChanged, false)

    // Confirm that WebGPU is actually supported.
    try {
        if (navigator.gpu === undefined) {
            throw new Error('`navigator.gpu` is undefined');
        }
        await navigator.gpu.requestAdapter();
    } catch (err) {
        console.error(err);

        romInput.disabled = true;

        const overlay = document.getElementById('screen-overlay')!;
        overlay.innerHTML = `
            <p>WebGPU is not enabled!</p>
            <p><a href="#enabling-webgpu">Please read the instructions below.</a></p>
        `;
        overlay.style.display = 'flex';

        return;
    }
}

const modalCloseButton = document.querySelector('#modal button') as HTMLElement;
modalCloseButton.onclick = () => {
    const modal = document.getElementById('modal')!;
    modal.style.display = 'none';
}

window.onclick = (e: MouseEvent) => {
    const modal = document.getElementById('modal')!;
    if (e.target === modal) {
        modal.style.display = "none";
    }
}

main();

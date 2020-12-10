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

    const romInput = document.getElementById('rom')!;
    romInput.addEventListener('change', romChanged, false)
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

import { Player } from "../pkg/fluster";

let sampleFileInput = document.getElementById("sample-file");
sampleFileInput.addEventListener("change", sampleFileSelected, false);

let localFileInput = document.getElementById("local-file");
localFileInput.addEventListener("change", localFileSelected, false);

let player;

function localFileSelected() {
    let file = localFileInput.files[0];
    if (file) {
        let fileReader = new FileReader();
        fileReader.onload = e => {
            playSwf(fileReader.result);
        }
        fileReader.readAsArrayBuffer(file);
    }
}

function sampleFileSelected() {
    if (sampleFileInput.selectedIndex <= 0) {
        // No SWF selected.
        return;
    }
    let file = sampleFileInput.selectedOptions[0].innerText;
    if (file) {
        fetch(file)
            .then(response => {
                response.arrayBuffer().then(data => playSwf(data))
            });
    }
}

let timestamp = 0;
let animationHandler;

function playSwf(swfData) {
    if (player) {
        player.destroy();
        window.cancelAnimationFrame(animationHandler);
        player = null;
        animationHandler = null;
    }

    let canvas = document.getElementById("player");
    if (swfData && canvas) {
        player = Player.new(canvas, new Uint8Array(swfData));
        timestamp = performance.now();
        animationHandler = window.requestAnimationFrame(tickPlayer);
    }
}

function tickPlayer(newTimestamp) {
    let dt = newTimestamp - timestamp;
    player.tick(dt);
    timestamp = newTimestamp;
    window.requestAnimationFrame(tickPlayer);
}
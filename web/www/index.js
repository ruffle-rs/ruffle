import { Player } from "../pkg/fluster";

let fileInput = document.getElementById("file-input");
fileInput.addEventListener("change", fileSelected, false);

let player;

function fileSelected() {
    let file = fileInput.files[0];
    if (file) {
        let fileReader = new FileReader();
        fileReader.onload = e => {
            playSwf(fileReader.result);
        }
        fileReader.readAsArrayBuffer(file);
    }
}

let timestamp = 0;
function playSwf(swfData) {
    let canvas = document.getElementById("fluster-canvas");
    if (swfData && canvas) {
        player = Player.new(canvas, new Uint8Array(swfData));
        timestamp = performance.now();
        window.requestAnimationFrame(tickPlayer);
    }
}

function tickPlayer(newTimestamp) {
    let dt = newTimestamp - timestamp;
    player.tick(dt);
    timestamp = newTimestamp;
    window.requestAnimationFrame(tickPlayer);
}
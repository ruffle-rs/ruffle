import { Player } from "fluster";

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

function playSwf(swfData) {
    let canvas = document.getElementById("fluster-canvas");
    if (swfData && canvas) {
        player = Player.new(swfData, canvas);
        window.requestAnimationFrame(tickPlayer);
    }
}

function tickPlayer(timestamp) {
    player.tick(timestamp);
    window.requestAnimationFrame(tickPlayer);
}
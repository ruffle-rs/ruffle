import { Player } from "../pkg/ruffle";

let sampleFileInput = document.getElementById("sample-file");
sampleFileInput.addEventListener("change", sampleFileSelected, false);

let localFileInput = document.getElementById("local-file");
localFileInput.addEventListener("change", localFileSelected, false);

let player;

if (window.location.search && window.location.search != "") {
    let urlParams = new URLSearchParams(window.location.search);
    let url = urlParams.get("file");
    if (url && url != "") {
        loadRemoteFile(url);
    }
}

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
        loadRemoteFile(file);
    }
}

function loadRemoteFile(url) {
    fetch(url)
        .then(response => {
            response.arrayBuffer().then(data => playSwf(data))
        });
}

let timestamp = 0;
let animationHandler;

function playSwf(swfData) {
    if (player) {
        player.destroy();
        player = null;
    }

    let canvas = document.getElementById("player");
    if (swfData && canvas) {
        player = Player.new(canvas, new Uint8Array(swfData));
    }
}
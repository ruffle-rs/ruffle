import "./index.css";

const { SourceAPI, PublicAPI } = require("ruffle-core");

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);

let ruffle;
let player;

const main = document.getElementById("main");
const overlay = document.getElementById("overlay");
const authorContainer = document.getElementById("author-container");
const author = document.getElementById("author");
const sampleFileInputContainer = document.getElementById(
    "sample-swfs-container"
);
const sampleFileInput = document.getElementById("sample-swfs");
const localFileInput = document.getElementById("local-file");
const animOptGroup = document.getElementById("anim-optgroup");
const gamesOptGroup = document.getElementById("games-optgroup");

// Default config used by the player.
const config = {
    letterbox: "on",
    logLevel: "warn",
};

function ensurePlayer() {
    if (player) {
        player.remove();
    }
    player = ruffle.createPlayer();
    player.id = "player";
    main.append(player);

    sampleFileInput.selectedIndex = 0;
    authorContainer.style.display = "none";
    author.textContent = "";
    author.href = "";
}

async function loadFile(file) {
    if (!file) {
        return;
    }
    ensurePlayer();
    const data = await new Response(file).arrayBuffer();
    player.load({ data, ...config });
}

function sampleFileSelected() {
    const swfData = sampleFileInput[sampleFileInput.selectedIndex].swfData;
    if (swfData) {
        authorContainer.style.display = "block";
        author.textContent = swfData.author;
        author.href = swfData.authorLink;
        localFileInput.value = null;
        player.load({ url: swfData.location, ...config });
    } else {
        ensurePlayer();
    }
}

localFileInput.addEventListener("change", (event) => {
    loadFile(event.target.files[0]);
});

sampleFileInput.addEventListener("change", sampleFileSelected);

main.addEventListener("dragenter", () => {
    overlay.classList.add("drag");
});
main.addEventListener("dragleave", () => {
    overlay.classList.remove("drag");
});
main.addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
});
main.addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
    loadFile(event.dataTransfer.files[0]);
});

window.addEventListener("load", () => {
    overlay.style.display = "initial";
});

window.addEventListener("DOMContentLoaded", async () => {
    ruffle = window.RufflePlayer.newest();
    ensurePlayer();

    const response = await fetch("swfs.json");
    if (!response.ok) {
        sampleFileInputContainer.style.display = "none";
        return;
    }

    const data = await response.json();
    for (const swfData of data.swfs) {
        const option = document.createElement("option");
        option.textContent = swfData.title;
        option.value = swfData.location;
        option.swfData = swfData;
        switch (swfData.type) {
            case "Animation":
                animOptGroup.append(option);
                break;
            case "Game":
                gamesOptGroup.append(option);
                break;
        }
    }
    sampleFileInputContainer.style.display = "inline-block";

    const initialFile = new URLSearchParams(window.location.search).get("file");
    if (initialFile) {
        const options = Array.from(sampleFileInput.options);
        sampleFileInput.selectedIndex = Math.max(
            options.findIndex((swfData) => swfData.value.endsWith(initialFile)),
            0
        );
        sampleFileSelected();
    }
});

import "./index.css";

const { SourceAPI, PublicAPI } = require("ruffle-core");

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);

let ruffle;
let player;

const container = document.getElementById("main");
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

window.addEventListener("DOMContentLoaded", async () => {
    ruffle = window.RufflePlayer.newest();
    player = ruffle.createPlayer();
    player.id = "player";
    container.append(player);

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
    } else {
        // Load a random file.
        sampleFileInput.selectedIndex =
            Math.floor(Math.random() * data.swfs.length) + 1;
    }
    sampleFileSelected();
});

window.addEventListener("load", () => {
    overlay.style.display = "block";
});

sampleFileInput.addEventListener("change", sampleFileSelected);
localFileInput.addEventListener("change", (event) => {
    loadFile(event.target.files[0]);
});
container.addEventListener("dragenter", () => {
    overlay.classList.add("drag");
});
container.addEventListener("dragleave", () => {
    overlay.classList.remove("drag");
});
container.addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
});
container.addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
    loadFile(event.dataTransfer.files[0]);
});

function sampleFileSelected() {
    const swfData = sampleFileInput[sampleFileInput.selectedIndex].swfData;
    if (swfData) {
        authorContainer.style.display = "block";
        author.textContent = swfData.author;
        author.href = swfData.authorLink;
        localFileInput.value = null;
        player.load({ url: swfData.location, ...config });
    } else {
        container.children[0].remove();
        player = ruffle.createPlayer();
        player.id = "player";
        container.append(player);
        authorContainer.style.display = "none";
        author.textContent = "";
        author.href = "";
    }
}

async function loadFile(file) {
    if (!file) {
        return;
    }

    sampleFileInput.selectedIndex = 0;
    authorContainer.style.display = "none";
    author.textContent = "";
    author.href = "";

    const data = await new Response(file).arrayBuffer();
    player.load({ data, ...config });
}

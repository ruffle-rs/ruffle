import "./index.css";

import { SourceAPI, PublicAPI } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);
const ruffle = window.RufflePlayer.newest();

let player;

const main = document.getElementById("main");
const overlay = document.getElementById("overlay");
const authorContainer = document.getElementById("author-container");
const author = document.getElementById("author");
const sampleFileInputContainer = document.getElementById(
    "sample-swfs-container"
);
const localFileInput = document.getElementById("local-file");
const sampleFileInput = document.getElementById("sample-swfs");
const localFileName = document.getElementById("local-file-name");
// prettier-ignore
const optionGroups = {
    "Animation": document.getElementById("anim-optgroup"),
    "Game": document.getElementById("games-optgroup"),
};

// Default config used by the player.
const defaultConfig = {
    letterbox: "on",
    logLevel: "warn",
};

function unload() {
    if (player) {
        player.remove();
    }
}

function load(options) {
    unload();
    player = ruffle.createPlayer();
    player.id = "player";
    main.append(player);
    player.load(options);
}

function showSample(swfData) {
    authorContainer.classList.remove("hidden");
    author.textContent = swfData.author;
    author.href = swfData.authorLink;
    localFileInput.value = null;
}

function hideSample() {
    sampleFileInput.selectedIndex = -1;
    authorContainer.classList.add("hidden");
    author.textContent = "";
    author.href = "";
}

async function loadFile(file) {
    if (!file) {
        return;
    }
    if (file.name) {
        localFileName.textContent = file.name;
    }
    hideSample();
    const data = await new Response(file).arrayBuffer();
    load({ data, ...defaultConfig });
}

function loadSample() {
    const swfData = sampleFileInput[sampleFileInput.selectedIndex].swfData;
    localFileName.textContent = "No file selected.";
    if (swfData) {
        showSample(swfData);
        const config = swfData.config || defaultConfig;
        load({ url: swfData.location, ...config });
    } else {
        hideSample();
        unload();
    }
}

localFileInput.addEventListener("change", (event) => {
    loadFile(event.target.files[0]);
});

sampleFileInput.addEventListener("change", () => loadSample());

main.addEventListener("dragenter", (event) => {
    event.stopPropagation();
    event.preventDefault();
});
main.addEventListener("dragleave", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
});
main.addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.add("drag");
});
main.addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
    localFileInput.files = event.dataTransfer.files;
    loadFile(event.dataTransfer.files[0]);
});
localFileInput.addEventListener("dragleave", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
});
localFileInput.addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.add("drag");
});
localFileInput.addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
    localFileInput.files = event.dataTransfer.files;
    loadFile(event.dataTransfer.files[0]);
});

window.addEventListener("load", () => {
    if (
        navigator.userAgent.match(/iPad/i) ||
        navigator.userAgent.match(/iPhone/i)
    ) {
        localFileInput.removeAttribute("accept");
    }
    overlay.classList.remove("hidden");
});

(async () => {
    const response = await fetch("swfs.json");

    if (response.ok) {
        const data = await response.json();
        for (const swfData of data.swfs) {
            const option = document.createElement("option");
            option.textContent = swfData.title;
            option.value = swfData.location;
            option.swfData = swfData;
            if (swfData.type) {
                optionGroups[swfData.type].append(option);
            } else {
                sampleFileInput.insertBefore(
                    option,
                    sampleFileInput.firstChild
                );
            }
        }
        sampleFileInputContainer.classList.remove("hidden");
    }

    sampleFileInput.selectedIndex = 0;
    const initialFile = new URL(window.location).searchParams.get("file");
    if (initialFile) {
        const options = Array.from(sampleFileInput.options);
        sampleFileInput.selectedIndex = Math.max(
            options.findIndex((swfData) => swfData.value.endsWith(initialFile)),
            0
        );
    }
    loadSample();
})();

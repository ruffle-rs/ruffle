import "./index.css";

import { PublicAPI } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(window.RufflePlayer, "local");
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

const swfToFlashVersion = {
    1: "FP1",
    2: "FP2",
    3: "FP3",
    4: "FP4",
    5: "FP5",
    6: "FP6",
    7: "FP7",
    8: "FP8",
    9: "FP9.0",
    10: "FP10.0 or FP10.1",
    11: "FP10.2",
    12: "FP10.3",
    13: "FP11.0",
    14: "FP11.1",
    15: "FP11.2",
    16: "FP11.3",
    17: "FP11.4",
    18: "FP11.5",
    19: "FP11.6",
    20: "FP11.7",
    21: "FP11.8",
    22: "FP11.9",
    23: "FP12",
    24: "FP13",
    25: "FP14",
    26: "FP15",
    27: "FP16",
    28: "FP17",
    29: "FP18",
    30: "FP19",
    31: "FP20",
    32: "FP21",
    33: "FP22",
    34: "FP23",
    35: "FP24",
    36: "FP25",
    37: "FP26",
    38: "FP27",
    39: "FP28",
    40: "FP29",
    41: "FP30",
    42: "FP31",
    43: "FP32",
}

function unload() {
    if (player) {
        player.remove();
        document.getElementById("flashversion").textContent = "loading";
        document.getElementById("asversion").textContent = "loading";
        document.getElementById("frameRate").textContent = "loading";
        document.getElementById("filesize").textContent = "loading";
        document.getElementById("width").textContent = "loading";
        document.getElementById("height").textContent = "loading";
        document.getElementById("numFrames").textContent = "loading";
        document.getElementById("backgroundColor").textContent = "loading";
    }
}

function load(options) {
    unload();
    player = ruffle.createPlayer();
    player.id = "player";
    main.append(player);
    player.load(options);
    player.addEventListener("loadedmetadata", function() {
        if (this.metadata) {
            if (this.metadata.swfVersion) {
                document.getElementById("flashversion").textContent = swfToFlashVersion[this.metadata.swfVersion];
            }
            if (this.metadata.isActionScript3) {
                document.getElementById("asversion").textContent = "yes";
            } else {
                document.getElementById("asversion").textContent = "no";
            }
            if (this.metadata.frameRate) {
                document.getElementById("frameRate").textContent = `${this.metadata.frameRate} FPS`;
            }
            if (this.metadata.width) {
                document.getElementById("width").textContent = `${this.metadata.width}px`;
            }
            if (this.metadata.height) {
                document.getElementById("height").textContent = `${this.metadata.height}px`;
            }
            if (this.metadata.numFrames) {
                document.getElementById("numFrames").textContent = this.metadata.numFrames;
            }
            if (this.metadata.backgroundColor) {
                document.getElementById("backgroundColor").textContent = this.metadata.backgroundColor;
            }
        }
    });
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
    if (file.size) {
        document.getElementById("filesize").textContent = `${Math.round(file.size/1024)}Kb`;
    }
}

function getFileSize(url, callback) {
    var xhr = new XMLHttpRequest();
    xhr.open("HEAD", url, true);
    xhr.onreadystatechange = function() {
        if (this.readyState == this.DONE) {
            document.getElementById("filesize").textContent = `${Math.round(parseInt(xhr.getResponseHeader("Content-Length"))/1024)}Kb`;
        }
    };
    xhr.send();
}

function loadSample() {
    const swfData = sampleFileInput[sampleFileInput.selectedIndex].swfData;
    localFileName.textContent = "No file selected.";
    if (swfData) {
        getFileSize(swfData.location);
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

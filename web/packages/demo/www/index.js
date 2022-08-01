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
const closeModal = document.getElementById("closeModal");
const openModal = document.getElementById("openModal");
const metadataModal = document.getElementById("metadataModal");
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
    1: "1",
    2: "2",
    3: "3",
    4: "4",
    5: "5",
    6: "6",
    7: "7",
    8: "8",
    9: "9.0",
    10: "10.0/10.1",
    11: "10.2",
    12: "10.3",
    13: "11.0",
    14: "11.1",
    15: "11.2",
    16: "11.3",
    17: "11.4",
    18: "11.5",
    19: "11.6",
    20: "11.7",
    21: "11.8",
    22: "11.9",
    23: "12",
    24: "13",
    25: "14",
    26: "15",
    27: "16",
    28: "17",
    29: "18",
    30: "19",
    31: "20",
    32: "21",
    33: "22",
    34: "23",
    35: "24",
    36: "25",
    37: "26",
    38: "27",
    39: "28",
    40: "29",
    41: "30",
    42: "31",
    43: "32",
};

function unload() {
    if (player) {
        player.remove();
        document.querySelectorAll("span.metadata").forEach((el) => {
            el.textContent = "Loading";
        });
        document.getElementById("backgroundColor").value = "#FFFFFF";
    }
}

function load(options) {
    unload();
    player = ruffle.createPlayer();
    player.id = "player";
    main.append(player);
    player.load(options);
    player.addEventListener("loadedmetadata", function () {
        if (this.metadata) {
            Object.keys(this.metadata).forEach((el) => {
                const metadataElement = document.getElementById(el);
                if (metadataElement) {
                    if (el === "backgroundColor") {
                        metadataElement.value = this.metadata[el] ?? "#FFFFFF";
                    } else {
                        if (el === "swfVersion") {
                            document.getElementById(
                                "flashVersion"
                            ).textContent =
                                swfToFlashVersion[this.metadata[el]];
                        }
                        metadataElement.textContent = this.metadata[el] ?? "?";
                    }
                }
            });
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

closeModal.addEventListener("click", () => {
    metadataModal.style.display = "none";
});

openModal.addEventListener("click", () => {
    metadataModal.style.display = "block";
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

window.onclick = (event) => {
    if (event.target === metadataModal) {
        metadataModal.style.display = "none";
    }
};

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

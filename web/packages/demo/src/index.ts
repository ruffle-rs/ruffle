import "./lato.css";
import "./common.css";
import "./index.css";

declare global {
    interface Navigator {
        /**
         * iPadOS sends a User-Agent string that appears to be from macOS.
         * navigator.standalone is not defined on macOS, so we use it for iPad detection.
         */
        standalone?: boolean;
    }
}

import {
    BaseLoadOptions,
    DataLoadOptions,
    Letterbox,
    LogLevel,
    PublicAPI,
    RufflePlayer,
    URLLoadOptions,
} from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(window.RufflePlayer, "local");
const ruffle = (window.RufflePlayer as PublicAPI).newest()!;

let player: RufflePlayer | null;

const playerContainer = document.getElementById("player-container")!;
const overlay = document.getElementById("overlay")!;
const authorContainer = document.getElementById("author-container")!;
const author = document.getElementById("author") as HTMLLinkElement;
const webUrlInputContainer = document.getElementById("web-url-container")!;
const sampleFileInputContainer = document.getElementById(
    "sample-swfs-container",
)!;
const localFileInput = document.getElementById(
    "local-file",
) as HTMLInputElement;
const sampleFileInput = document.getElementById(
    "sample-swfs",
) as HTMLSelectElement;
const localFileName = document.getElementById("local-file-name")!;
const toggleInfo = document.getElementById("toggle-info")!;
const reloadSwf = document.getElementById("reload-swf")!;
const infoContainer = document.getElementById("info-container")!;
// prettier-ignore
const optionGroups = {
    "Animation": document.getElementById("anim-optgroup")!,
    "Game": document.getElementById("games-optgroup")!,
};

// This is the base config used by the demo player (except for specific SWF files
// with their own base config).
// It has the highest priority and its options cannot be overwritten.
const baseDemoConfig = {
    letterbox: Letterbox.On,
    logLevel: LogLevel.Warn,
    forceScale: true,
    forceAlign: true,
};

// [NA] For when we consolidate the extension player, and the web demo
const enableUrlInput = false;

if (!enableUrlInput) {
    webUrlInputContainer.classList.add("hidden");
}

const swfToFlashVersion: { [key: number]: string } = {
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

interface DemoSwf {
    location: string;
    title?: string;
    author?: string;
    authorLink?: string;
    config?: BaseLoadOptions;
    type: "Animation" | "Game";
}

interface HTMLOptionElementWithSwf extends HTMLOptionElement {
    swfData: DemoSwf;
}

function unload() {
    if (player) {
        player.remove();
        document.querySelectorAll("span.metadata").forEach((el) => {
            el.textContent = "Loading";
        });
        document.getElementById("backgroundColor")!.style.backgroundColor =
            "white";
    }
}

function load(options: string | DataLoadOptions | URLLoadOptions) {
    unload();
    player = ruffle.createPlayer();
    player.id = "player";
    playerContainer.append(player);
    player.load(options, false);
    player.addEventListener("loadedmetadata", () => {
        if (player?.metadata) {
            for (const [key, value] of Object.entries(player.metadata)) {
                const metadataElement = document.getElementById(key);
                if (metadataElement) {
                    switch (key) {
                        case "backgroundColor":
                            metadataElement.style.backgroundColor =
                                value ?? "white";
                            break;
                        case "uncompressedLength":
                            metadataElement.textContent = `${value >> 10}Kb`;
                            break;
                        // @ts-expect-error This intentionally falls through to the default case
                        case "swfVersion":
                            document.getElementById(
                                "flashVersion",
                            )!.textContent = swfToFlashVersion[value] ?? null;
                        // falls through and executes the default case as well
                        default:
                            metadataElement.textContent = value;
                            break;
                    }
                }
            }
        }
    });
}

function showSample(swfData: DemoSwf) {
    authorContainer.classList.remove("hidden");
    author.textContent = swfData.author ?? "Unknown";
    author.href = swfData.authorLink ?? "#";
    localFileInput.value = "";
}

function hideSample() {
    sampleFileInput.selectedIndex = -1;
    authorContainer.classList.add("hidden");
    author.textContent = "";
    author.href = "";
}

async function loadFile(file: File | undefined) {
    if (!file) {
        return;
    }
    if (file.name) {
        localFileName.textContent = file.name;
    }
    hideSample();
    const data = await new Response(file).arrayBuffer();
    load({ data: data, swfFileName: file.name, ...baseDemoConfig });
}

function loadSample() {
    const swfData = (
        sampleFileInput[
            sampleFileInput.selectedIndex
        ] as HTMLOptionElementWithSwf
    ).swfData;
    localFileName.textContent = "No file selected.";
    if (swfData) {
        showSample(swfData);
        const config = swfData.config || baseDemoConfig;
        load({ url: swfData.location, ...config });
    } else {
        hideSample();
        unload();
    }
}

localFileInput.addEventListener("change", (event) => {
    const eventTarget = event.target as HTMLInputElement;
    if (
        eventTarget?.files &&
        eventTarget?.files.length > 0 &&
        eventTarget.files[0]
    ) {
        loadFile(eventTarget.files[0]);
    }
});

sampleFileInput.addEventListener("change", () => loadSample());

playerContainer.addEventListener("dragenter", (event) => {
    event.stopPropagation();
    event.preventDefault();
});
playerContainer.addEventListener("dragleave", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
});
playerContainer.addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.add("drag");
});
playerContainer.addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();
    overlay.classList.remove("drag");
    if (event.dataTransfer) {
        localFileInput.files = event.dataTransfer.files;
        loadFile(event.dataTransfer.files[0]);
    }
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
    if (event.dataTransfer) {
        localFileInput.files = event.dataTransfer.files;
        loadFile(event.dataTransfer.files[0]);
    }
});

toggleInfo.addEventListener("click", () => {
    if (infoContainer.style.display === "none") {
        infoContainer.style.display = "flex";
    } else {
        infoContainer.style.display = "none";
    }
});

reloadSwf.addEventListener("click", () => {
    if (player) {
        const confirmReload = confirm("Reload the current SWF?");
        if (confirmReload) {
            player.reload();
        }
    }
});

window.addEventListener("load", () => {
    if (
        navigator.userAgent.match(/iPad/i) ||
        navigator.userAgent.match(/iPhone/i) ||
        (navigator.platform === "MacIntel" &&
            typeof navigator.standalone !== "undefined")
    ) {
        localFileInput.removeAttribute("accept");
    }
    overlay.removeAttribute("hidden");
});

(async () => {
    const response = await fetch("swfs.json");

    if (response.ok) {
        const data: { swfs: [DemoSwf] } = await response.json();
        for (const swfData of data.swfs) {
            const option = document.createElement(
                "option",
            ) as HTMLOptionElementWithSwf;
            option.textContent = swfData.title ?? "Unknown";
            option.value = swfData.location;
            option.swfData = swfData;
            if (swfData.type) {
                optionGroups[swfData.type].append(option);
            } else {
                sampleFileInput.insertBefore(
                    option,
                    sampleFileInput.firstChild,
                );
            }
        }
        sampleFileInputContainer.classList.remove("hidden");
    }

    sampleFileInput.selectedIndex = 0;
    const initialFile = new URL(window.location.href).searchParams.get("file");
    if (initialFile) {
        const options = Array.from(sampleFileInput.options);
        sampleFileInput.selectedIndex = Math.max(
            options.findIndex((swfData) => swfData.value.endsWith(initialFile)),
            0,
        );
    }
    loadSample();
})();

document.getElementById("local-file-label")!.addEventListener("click", () => {
    document.getElementById("local-file")!.click();
});

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

const main = document.getElementById("main")!;
const overlay = document.getElementById("overlay")!;
const authorContainer = document.getElementById("author-container")!;
const author = <HTMLLinkElement>document.getElementById("author");
const sampleFileInputContainer = document.getElementById(
    "sample-swfs-container",
)!;
const localFileInput = <HTMLInputElement>document.getElementById("local-file");
const sampleFileInput = <HTMLSelectElement>(
    document.getElementById("sample-swfs")
);
const localFileName = document.getElementById("local-file-name")!;
const closeModal = document.getElementById("close-modal")!;
const openModal = document.getElementById("open-modal")!;
const reloadSwf = document.getElementById("reload-swf")!;
const metadataModal = document.getElementById("metadata-modal")!;
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

const swfToFlashVersion: Record<number, string> = {
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
        (<HTMLInputElement>document.getElementById("backgroundColor")).value =
            "#FFFFFF";
    }
}

function load(options: DataLoadOptions | URLLoadOptions) {
    unload();
    player = ruffle.createPlayer();
    player.id = "player";
    main.append(player);
    player.load(options, false);
    player.addEventListener("loadedmetadata", function () {
        if (player?.metadata) {
            for (const [key, value] of Object.entries(player.metadata)) {
                const metadataElement = document.getElementById(key);
                if (metadataElement) {
                    switch (key) {
                        case "backgroundColor":
                            (<HTMLInputElement>metadataElement).value =
                                value ?? "#FFFFFF";
                            break;
                        case "uncompressedLength":
                            metadataElement.textContent = `${value >> 10}Kb`;
                            break;
                        case "swfVersion":
                            document.getElementById(
                                "flashVersion",
                            )!.textContent =
                                swfToFlashVersion[value] ?? "Unknown";
                            metadataElement.textContent = value;
                            break;
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

async function loadFile(file: File) {
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
    const swfData = (<HTMLOptionElementWithSwf>(
        sampleFileInput[sampleFileInput.selectedIndex]
    )).swfData;
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

localFileInput.addEventListener("change", (_event) => {
    if (localFileInput.files && localFileInput.files[0]) {
        loadFile(localFileInput.files[0]);
    }
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
    localFileInput.files = event.dataTransfer?.files ?? null;
    if (event.dataTransfer?.files[0]) {
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
    localFileInput.files = event.dataTransfer?.files ?? null;
    if (event.dataTransfer?.files[0]) {
        loadFile(event.dataTransfer.files[0]);
    }
});

closeModal.addEventListener("click", () => {
    metadataModal.style.display = "none";
});

openModal.addEventListener("click", () => {
    metadataModal.style.display = "block";
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
        const data: { swfs: [DemoSwf] } = await response.json();
        for (const swfData of data.swfs) {
            const option = <HTMLOptionElementWithSwf>(
                document.createElement("option")
            );
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

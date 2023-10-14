import * as utils from "./utils";
import { PublicAPI } from "ruffle-core";
import type {
    Letterbox,
    RufflePlayer,
    DataLoadOptions,
    URLLoadOptions,
} from "ruffle-core";

const api = PublicAPI.negotiate(window.RufflePlayer!, "local");
window.RufflePlayer = api;
const ruffle = api.newest()!;
let player: RufflePlayer;

const main = document.getElementById("main")!;
const overlay = document.getElementById("overlay")!;
const localFileInput = document.getElementById(
    "local-file",
)! as HTMLInputElement;
const localFileName = document.getElementById("local-file-name")!;
const closeModal = document.getElementById("close-modal")!;
const openModal = document.getElementById("open-modal")!;
const reloadSwf = document.getElementById("reload-swf")!;
const metadataModal = document.getElementById("metadata-modal")!;
const webFormSubmit = document.getElementById("web-form-submit")!;
const webURL = document.getElementById("web-url")! as HTMLInputElement;

// Default config used by the player.
const defaultConfig = {
    letterbox: "on" as Letterbox,
    forceScale: true,
    forceAlign: true,
};

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

function unload() {
    if (player) {
        player.remove();
        document.querySelectorAll("span.metadata").forEach((el) => {
            el.textContent = "Loading";
        });
        (
            document.getElementById("backgroundColor")! as HTMLInputElement
        ).value = "#FFFFFF";
    }
}

function load(options: string | DataLoadOptions | URLLoadOptions) {
    unload();
    player = ruffle.createPlayer();
    player.id = "player";
    main.append(player);
    player.load(options);
    player.addEventListener("loadedmetadata", () => {
        if (player.metadata) {
            for (const [key, value] of Object.entries(player.metadata)) {
                const metadataElement = document.getElementById(key);
                if (metadataElement) {
                    switch (key) {
                        case "backgroundColor":
                            (metadataElement as HTMLInputElement).value =
                                value ?? "#FFFFFF";
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

async function loadFile(file: File | undefined) {
    if (!file) {
        return;
    }
    if (file.name) {
        localFileName.textContent = file.name;
    }
    const data = await new Response(file).arrayBuffer();
    load({ data: data, swfFileName: file.name, ...defaultConfig });
    history.pushState("", document.title, window.location.pathname);
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

async function loadSwf(swfUrl: string) {
    try {
        const pathname = new URL(swfUrl).pathname;
        document.title = pathname.substring(pathname.lastIndexOf("/") + 1);
    } catch (_) {
        // Ignore URL parsing errors.
    }

    const options = await utils.getExplicitOptions();
    localFileName.textContent = document.title;
    localFileInput.value = "";
    load({
        ...options,
        url: swfUrl,
        base: swfUrl.substring(0, swfUrl.lastIndexOf("/") + 1),
        ...defaultConfig,
    });
}

async function loadSwfFromHash() {
    const url = new URL(window.location.href);
    // Hash always starts with #, gotta slice that off
    const swfUrl = url.hash.length > 1 ? url.hash.slice(1) : null;
    if (swfUrl) {
        webURL.value = swfUrl;
        await loadSwf(swfUrl);
    }
}

window.addEventListener("pageshow", loadSwfFromHash);

window.addEventListener("hashchange", loadSwfFromHash);

window.addEventListener("DOMContentLoaded", () => {
    webFormSubmit.addEventListener("click", () => {
        if (webURL.value !== "") {
            window.location.hash = webURL.value;
        }
    });
    webURL.addEventListener("keydown", (event) =>
        event.key === "Enter" ? webFormSubmit.click() : undefined,
    );
});

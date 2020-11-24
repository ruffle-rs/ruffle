const { SourceAPI, PublicAPI } = require("ruffle-core");

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);

let ruffle;
let player;
let jsonData;
let initialFile;

let container = document.getElementById("main");
let author_container = document.getElementById("author-container");
let author = document.getElementById("author");
let sampleFileInputContainer = document.getElementById("sample-swfs-container");
let sampleFileInput = document.getElementById("sample-swfs");
let localFileInput = document.getElementById("local-file");
let animOptGroup = document.getElementById("anim-optgroup");
let gamesOptGroup = document.getElementById("games-optgroup");

window.addEventListener("DOMContentLoaded", () => {
    ruffle = window.RufflePlayer.newest();
    player = ruffle.createPlayer();
    player.id = "player";
    container.appendChild(player);
    initialFile = new URLSearchParams(window.location.search).get("file");
    fetch("swfs.json").then((response) => {
        if (response.ok) {
            response.json().then((data) => {
                jsonData = data;
                jsonData.swfs.forEach((item) => {
                    let temp = document.createElement("option");
                    temp.innerHTML = item.title;
                    temp.setAttribute("value", item.location);
                    temp.swfData = item;
                    if (item.type == "Animation") {
                        animOptGroup.append(temp);
                    } else if (item.type == "Game") {
                        gamesOptGroup.appendChild(temp);
                    }
                });
                sampleFileInputContainer.style.display = "inline-block";

                if (initialFile) {
                    let opts = Array.from(sampleFileInput.options).map(
                        (item) => item.value
                    );
                    sampleFileInput.selectedIndex = Math.max(
                        opts.findIndex((item) => item.endsWith(initialFile)),
                        0
                    );
                } else {
                    // Load a random file.
                    let rn = Math.floor(
                        Math.random() * Math.floor(jsonData.swfs.length)
                    );
                    sampleFileInput.selectedIndex = rn + 1;
                }
                sampleFileSelected();
            });
        } else {
            sampleFileInputContainer.style.display = "none";
        }
    });
});

if (sampleFileInput) {
    sampleFileInput.addEventListener("change", sampleFileSelected, false);
}

if (localFileInput) {
    localFileInput.addEventListener("change", localFileSelected, false);
}

function sampleFileSelected() {
    let swfData = sampleFileInput[sampleFileInput.selectedIndex].swfData;
    if (swfData) {
        author_container.style.display = "block";
        author.innerHTML = swfData.author;
        author.href = swfData.authorLink;
        localFileInput.value = null;
        loadRemoteFile(swfData.location);
    } else {
        replacePlayer();
    }
}

function localFileSelected() {
    sampleFileInput.selectedIndex = 0;
    author_container.style.display = "none";
    author.innerHTML = "";
    author.href = "";

    let file = localFileInput.files[0];
    if (file) {
        let fileReader = new FileReader();
        fileReader.onload = () => {
            player.playSwfData(fileReader.result);
        };
        fileReader.readAsArrayBuffer(file);
    }
}

function loadRemoteFile(url) {
    fetch(url).then((response) => {
        response.arrayBuffer().then((data) => player.playSwfData(data));
    });
}

function replacePlayer() {
    document.getElementById("main").children[0].remove();
    player = ruffle.create_player();
    player.id = "player";
    container.appendChild(player);
    author_container.style.display = "none";
    author.innerHTML = "";
    author.href = "";
}

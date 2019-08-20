import { Ruffle } from "../../pkg/ruffle";

let ruffle_tmpl = document.createElement("template");
ruffle_tmpl.innerHTML = `
    <canvas id="player"></canvas>
`;

class RuffleObjectShadow {
    constructor(elem) {
        this.params = RuffleObjectShadow.params_of(elem);
        this.oldelem = elem;
        this.elem = document.createElement("div");
        this.shadow = this.elem.attachShadow({mode: 'closed'});
        this.shadow.appendChild(ruffle_tmpl.content.cloneNode(true));

        this.canvas = this.shadow.getElementById("player");
        this.ruffle = null;

        //Swap elements around.
        elem.parentElement.replaceChild(this.elem, elem);

        //Kick off the SWF download.
        if (this.params.movie) {
            fetch(this.params.movie).then(response => {
                response.arrayBuffer().then(data => this.play_swf(data))
            });
        }
    }

    play_swf(data) {
        if (this.ruffle) {
            this.ruffle.destroy();
            this.ruffle = null;
        }

        this.ruffle = Ruffle.new(this.canvas, new Uint8Array(data));
    }

    static params_of(elem) {
        let params = {};

        for (var param in elem.children) {
            if (param.constructor === HTMLParamElement) {
                params[param.name] = param.value;
            }
        }

        return params;
    }

    static wrap_tree(elem) {
        for (let node of elem.getElementsByTagName("object")) {
            if (node.type === "application/x-shockwave-flash") {
                new RuffleObjectShadow(node);
            }
        }
    }
}

const observer = new MutationObserver(function (mutationsList, observer) {
    for (let mutation of mutationsList) {
        for (let node of mutation.addedNodes) {
            RuffleObjectShadow.wrap_tree(node);
        }
    }
});

RuffleObjectShadow.wrap_tree(document);
observer.observe(document, { childList: true, subtree: true});

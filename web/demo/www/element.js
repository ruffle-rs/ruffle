import { Ruffle } from "../../pkg/ruffle";

let ruffle_tmpl = document.createElement("template");
ruffle_tmpl.innerHTML = `
    <canvas id="player"></canvas>
`;

class RuffleObjectShadow extends HTMLElement {
    constructor() {
        super();

        this.params = RuffleObjectShadow.params_of(this);

        this.shadow = this.attachShadow({mode: 'closed'});
        this.shadow.appendChild(ruffle_tmpl.content.cloneNode(true));

        this.canvas = this.shadow.getElementById("player");
        this.ruffle = null;
    }

    connected() {
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
                RuffleObjectShadow.replace_native_object(node);
            }
        }
    }

    static replace_native_object(elem) {
        var ruffle_obj = document.createElement("ruffle-object");
        for (let attrib of elem.attributes) {
            if (attrib.specified) {
                ruffle_obj.setAttribute(attrib.name, attrib.value);
            }
        }

        for (let node of elem.children) {
            ruffle_obj.appendChild(node);
        }

        //TODO: Preserve event handlers.

        //Swap elements around.
        //
        //Unfortunately, this isn't entirely transparent: Sites that have
        //manually created a Flash tag and want to communicate with it might
        //hold a reference to the old, disconnected object. We need to intercept
        //that kind of usage somehow.
        //
        //If JS tries to grab the object from the DOM again, then we're fine,
        //and we can present ourselves as a perfectly normal object tag in every
        //way except `nodeName`.
        elem.parentElement.replaceChild(ruffle_obj, elem);
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

window.customElements.define("ruffle-object", RuffleObjectShadow);
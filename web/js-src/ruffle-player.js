import { Ruffle } from "../pkg/ruffle";
import ruffle_shadow_template from "./shadow-template";

export default class RufflePlayer extends HTMLElement {
    constructor(...args) {
        let self = super(...args);

        self.shadow = self.attachShadow({mode: 'closed'});
        self.shadow.appendChild(ruffle_shadow_template.content.cloneNode(true));

        self.canvas = self.shadow.getElementById("player");
        self.ruffle = null;

        return self;
    }

    stream_swf_url(url) {
        //TODO: Actually stream files...
        console.log("Loading SWF file " + url);
        return fetch(url).then(response => {
            if (response.ok) {
                response.arrayBuffer().then(data => this.play_swf_data(data))
            } else {
                console.error("SWF load failed: " + response.status + " " + response.statusText + " for " + url);
            }
        });
    }

    play_swf_data(data) {
        if (this.ruffle) {
            this.ruffle.destroy();
            this.ruffle = null;
        }

        this.ruffle = Ruffle.new(this.canvas, new Uint8Array(data));
    }
}
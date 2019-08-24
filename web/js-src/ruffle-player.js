import load_ruffle from "./load-ruffle";
import ruffle_shadow_template from "./shadow-template";

export default class RufflePlayer extends HTMLElement {
    constructor(...args) {
        let self = super(...args);

        self.shadow = self.attachShadow({mode: 'closed'});
        self.shadow.appendChild(ruffle_shadow_template.content.cloneNode(true));

        self.canvas = self.shadow.getElementById("player");
        self.instance = null;

        self.Ruffle = load_ruffle();

        return self;
    }

    async stream_swf_url(url) {
        //TODO: Actually stream files...
        try {
            let abs_url = new URL(url, window.location.href).toString();
            console.log("Loading SWF file " + url);

            let response = await fetch(abs_url);

            if (response.ok) {
                let data = await response.arrayBuffer();
                await this.play_swf_data(data);
                console.log("Playing " + url);
            } else {
                console.error("SWF load failed: " + response.status + " " + response.statusText + " for " + url);
            }
        } catch (err) {
            console.error("Serious error occured loading SWF file: " + err);
            throw err;
        }
    }

    async play_swf_data(data) {
        console.log("Got SWF data");

        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
        }

        let Ruffle = await this.Ruffle.catch(function (e) {
            console.error("Serious error loading Ruffle: " + e);
            throw e;
        });
        
        this.instance = Ruffle.new(this.canvas, new Uint8Array(data));
    }
}
const load_ruffle = require("./load-ruffle");
const ruffle_shadow_template = require("./shadow-template");
const { lookup_element } = require("./register-element");

exports.FLASH_MIMETYPE = "application/x-shockwave-flash";
exports.FUTURESPLASH_MIMETYPE = "application/futuresplash";
exports.FLASH7_AND_8_MIMETYPE = "application/x-shockwave-flash2-preview";
exports.FLASH_MOVIE_MIMETYPE = "application/vnd.adobe.flash-movie";
exports.FLASH_ACTIVEX_CLASSID = "clsid:D27CDB6E-AE6D-11cf-96B8-444553540000";

const DIMENSION_REGEX = /^\s*(\d+(\.\d+)?(%)?)/;

function sanitize_parameters(parameters) {
    if (parameters === null || parameters === undefined) {
        return {};
    }
    if (!(parameters instanceof URLSearchParams)) {
        parameters = new URLSearchParams(parameters);
    }
    const output = {};

    for (const [key, value] of parameters) {
        // Every value must be type of string
        output[key] = value.toString();
    }

    return output;
}

exports.RufflePlayer = class RufflePlayer extends HTMLElement {
    constructor(...args) {
        let self = super(...args);

        self.shadow = self.attachShadow({ mode: "closed" });
        self.shadow.appendChild(ruffle_shadow_template.content.cloneNode(true));

        self.dynamic_styles = self.shadow.getElementById("dynamic_styles");
        self.container = self.shadow.getElementById("container");
        self.play_button = self.shadow.getElementById("play_button");
        if (self.play_button) {
            self.play_button.addEventListener(
                "click",
                self.play_button_clicked.bind(self)
            );
        }
        self.unmute_overlay = self.shadow.getElementById("unmute_overlay");
        if (self.unmute_overlay) {
            self.unmute_overlay.addEventListener(
                "click",
                self.unmute_overlay_clicked.bind(self)
            );
        }

        self.instance = null;
        self.allow_script_access = false;
        self.user_config = null;
        self._applied_config = null;
        self._trace_observer = null;

        self.Ruffle = load_ruffle();

        return self;
    }

    set_user_config() {
        // Set the default config.
        let config = (this._applied_config = {
            autoplay: "auto",
            displayUnmuteOverlay: "on",
        });

        // Get the global user config if specified.
        let global_player = window.RufflePlayer;
        if (global_player && global_player.config) {
            let global_config = global_player.config;
            for (let option in config) {
                if (
                    Object.prototype.hasOwnProperty.call(config, option) &&
                    Object.prototype.hasOwnProperty.call(global_config, option)
                ) {
                    config[option] = global_config[option];
                }
            }
        }

        // Get the user config for the current file if specified, this is only possible using the JS API.
        if (this.user_config !== null) {
            let user_config = this.user_config;
            for (let option in config) {
                if (
                    Object.prototype.hasOwnProperty.call(config, option) &&
                    Object.prototype.hasOwnProperty.call(user_config, option)
                ) {
                    config[option] = user_config[option];
                }
            }
        }

        // Apply the config to the different options.
        this.set_autoplay_config(config.autoplay);
        this.set_unmuteoverlay_config(config.displayUnmuteOverlay);
    }

    set_autoplay_config(behavior) {
        let applied_config = this._applied_config;
        applied_config.autoplay = false;
        switch (behavior) {
            case "auto":
            default:
                if (this.audio_state() === "running") {
                    applied_config.autoplay = true;
                }
                break;
            case "on":
                applied_config.autoplay = true;
                break;
            case "off":
                break;
        }
    }

    set_unmuteoverlay_config(behavior) {
        let applied_config = this._applied_config;
        applied_config.displayUnmuteOverlay = false;
        switch (behavior) {
            case "on":
            default:
                if (
                    this.unmute_overlay &&
                    applied_config.autoplay &&
                    this.audio_state() !== "running"
                ) {
                    this.unmute_overlay.style.display = "block";
                    let self = this;
                    let audio_context = this.instance && this.instance.audio_context();
                    if (audio_context) {
                        audio_context.onstatechange = function () {
                            if (this.state === "running") {
                                self.unmute_overlay_clicked();
                            }
                            this.onstatechange = null;
                        };
                    }
                }
                break;
            case "off":
                break;
        }
    }

    connectedCallback() {
        this.update_styles();
    }

    static get observedAttributes() {
        return ["width", "height"];
    }

    attributeChangedCallback(name) {
        if (name === "width" || name === "height") {
            this.update_styles();
        }
    }

    disconnectedCallback() {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }
    }

    update_styles() {
        if (this.dynamic_styles.sheet) {
            if (this.dynamic_styles.sheet.rules) {
                for (
                    var i = 0;
                    i < this.dynamic_styles.sheet.rules.length;
                    i++
                ) {
                    this.dynamic_styles.sheet.deleteRule(i);
                }
            }

            if (this.attributes.width) {
                let width = RufflePlayer.html_dimension_to_css_dimension(
                    this.attributes.width.value
                );
                if (width !== null) {
                    this.dynamic_styles.sheet.insertRule(
                        `:host { width: ${width}; }`
                    );
                }
            }

            if (this.attributes.height) {
                let height = RufflePlayer.html_dimension_to_css_dimension(
                    this.attributes.height.value
                );
                if (height !== null) {
                    this.dynamic_styles.sheet.insertRule(
                        `:host { height: ${height}; }`
                    );
                }
            }
        }
    }

    /**
     * Determine if this element is the fallback content of another Ruffle
     * player.
     *
     * This heurustic assumes Ruffle objects will never use their fallback
     * content. If this changes, then this code also needs to change.
     */
    is_unused_fallback_object() {
        let parent = this.parentNode;
        let element = lookup_element("ruffle-object");

        if (element !== null) {
            do {
                if (parent.nodeName === element.name) {
                    return true;
                }

                parent = parent.parentNode;
            } while (parent != document);
        }

        return false;
    }

    /**
     * Ensure a fresh Ruffle instance is ready on this player before continuing.
     *
     * @throws Any exceptions generated by loading Ruffle Core will be logged
     * and passed on.
     */
    async ensure_fresh_instance() {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }

        let Ruffle = await this.Ruffle.catch((e) => {
            console.error("Serious error loading Ruffle: " + e);
            throw e;
        });

        this.instance = Ruffle.new(
            this.container,
            this,
            this.allow_script_access
        );
        if (this._applied_config.autoplay) {
            this.play_button_clicked(this);
        } else {
            if (this.play_button) {
                this.play_button.style.display = "block";
            }
        }
        console.log("New Ruffle instance created.");
    }

    /**
     * Load a movie into this Ruffle Player instance by URL.
     *
     * Any existing movie will be immediately stopped, while the new movie's
     * load happens asynchronously. There is currently no way to await the file
     * being loaded, or any errors that happen loading it.
     *
     * @param {String} url The URL to stream.
     * @param {URLSearchParams|String|Object} [parameters] The parameters (also known as "flashvars") to load the movie with.
     * If it's a string, it will be decoded into an object.
     * If it's an object, every key and value must be a String.
     * These parameters will be merged onto any found in the query portion of the swf URL.
     */
    async stream_swf_url(url, parameters) {
        //TODO: Actually stream files...
        try {
            if (this.isConnected && !this.is_unused_fallback_object()) {
                console.log("Loading SWF file " + url);

                this.set_user_config();
                await this.ensure_fresh_instance();
                parameters = {
                    ...sanitize_parameters(url.substring(url.indexOf("?"))),
                    ...sanitize_parameters(parameters),
                };
                this.instance.stream_from(url, parameters);
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error("Serious error occurred loading SWF file: " + err);
            this.panic(err);
            throw err;
        }
    }

    play_button_clicked() {
        if (this.instance) {
            this.instance.play();
            if (this.play_button) {
                this.play_button.style.display = "none";
            }
        }
    }

    pause() {
        if (this.instance) {
            this.instance.pause();
            if (this.play_button) {
                this.play_button.style.display = "block";
            }
        }
    }

    audio_state() {
        if (this.instance) {
            let audio_context = this.instance.audio_context();
            return (audio_context && audio_context.state) || "running";
        }
        return "suspended";
    }

    unmute_overlay_clicked() {
        if (this.instance) {
            if (this.audio_state() !== "running") {
                let audio_context = this.instance.audio_context();
                if (audio_context) {
                    audio_context.resume();
                }
            }
            if (this.unmute_overlay) {
                this.unmute_overlay.style.display = "none";
            }
        }
    }

    /**
     * Load a movie's data into this Ruffle Player instance.
     *
     * Any existing movie will be immediately stopped, and the new movie's data
     * placed into a fresh Stage on the same stack.
     *
     * Please note that by doing this, no URL information will be provided to
     * the movie being loaded.
     *
     * @param {String} data The data to decode.
     * @param {URLSearchParams|String|Object} [parameters] The parameters (also known as "flashvars") to load the movie with.
     * If it's a string, it will be decoded into an object.
     * If it's an object, every key and value must be a String.
     */
    async play_swf_data(data, parameters) {
        try {
            if (this.isConnected && !this.is_unused_fallback_object()) {
                console.log("Got SWF data");

                this.set_user_config();
                await this.ensure_fresh_instance();
                this.instance.load_data(
                    new Uint8Array(data),
                    sanitize_parameters(parameters)
                );
                console.log("New Ruffle instance created.");
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error("Serious error occurred loading SWF file: " + err);
            this.panic(err);
            throw err;
        }
    }

    /*
     * Copies attributes and children from another element to this player element.
     * Used by the polyfill elements, RuffleObject and RuffleEmbed.
     */
    copy_element(elem) {
        if (elem) {
            for (let attrib of elem.attributes) {
                if (attrib.specified) {
                    // Issue 468: Chrome "Click to Active Flash" box stomps on title attribute
                    if (
                        attrib.name === "title" &&
                        attrib.value === "Adobe Flash Player"
                    ) {
                        continue;
                    }

                    try {
                        this.setAttribute(attrib.name, attrib.value);
                    } catch (err) {
                        // The embed may have invalid attributes, so handle these gracefully.
                        console.warn(
                            `Unable to set attribute ${attrib.name} on Ruffle instance`
                        );
                    }
                }
            }

            for (let node of Array.from(elem.children)) {
                this.appendChild(node);
            }
        }
    }

    /*
     * Converts a dimension attribute on an HTML embed/object element to a valid CSS dimension.
     * HTML element dimensions are unitless, but can also be percentages.
     * Add a 'px' unit unless the value is a percentage.
     * Returns null if this is not a valid dimension.
     */
    static html_dimension_to_css_dimension(attribute) {
        if (attribute) {
            let match = attribute.match(DIMENSION_REGEX);
            if (match) {
                let out = match[1];
                if (!match[3]) {
                    // Unitless -- add px for CSS.
                    out += "px";
                }
                return out;
            }
        }
        return null;
    }

    /*
     * When a movie presents a new callback through `ExternalInterface.addCallback`,
     * we are informed so that we can expose the method on any relevant DOM element.
     */
    on_callback_available(name) {
        const instance = this.instance;
        this[name] = (...args) => {
            return instance.call_exposed_callback(name, args);
        };
    }

    /*
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     */
    set trace_observer(observer) {
        this.instance.set_trace_observer(observer);
    }

    /*
     * Panics this specific player, forcefully destroying all resources and displays an error message to the user.
     *
     * This should be called when something went absolutely, incredibly and disastrously wrong and there is no chance
     * of recovery.
     *
     * Ruffle will attempt to isolate all damage to this specific player instance, but no guarantees can be made if there
     * was a core issue which triggered the panic. If Ruffle is unable to isolate the cause to a specific player, then
     * all players will panic and Ruffle will become "poisoned" - no more players will run on this page until it is
     * reloaded fresh.
     */
    panic(error) {
        // Clears out any existing content (ie play button or canvas) and replaces it with the error screen
        this.container.innerHTML = `
            <div id="panic">
                <div id="panic-title">Something went wrong :(</div>
                <div id="panic-body">
                    <p>Ruffle has encountered a major issue whilst trying to display this Flash content.</p>
                    <p>This isn't supposed to happen, so we'd really appreciate if you could file a bug!</p>
                </div>
                <div id="panic-footer">
                    <ul>
                        <li><a href="https://github.com/ruffle-rs/ruffle/issues/new">report bug</a></li>
                        <li><a href="#" id="panic-view-details">view error details</a></li>
                    </ul>
                </div>
            </div>
        `;
        this.container.querySelector("#panic-view-details").onclick = () => {
            let error_text = "# Error Info\n";

            if (error instanceof Error) {
                error_text += `Error name: ${error.name}\n`;
                error_text += `Error message: ${error.message}\n`;
                if (error.stack) {
                    error_text += `Error stack:\n\`\`\`\n${error.stack}\n\`\`\`\n`;
                }
            } else {
                error_text += `Error: ${error}\n`;
            }

            error_text += "\n# Player Info\n";
            error_text += this.debug_player_info();

            error_text += "\n# Page Info\n";
            error_text += `Page URL: ${document.location.href}\n`;

            error_text += "\n# Browser Info\n";
            error_text += `Useragent: ${window.navigator.userAgent}\n`;
            error_text += `OS: ${window.navigator.platform}\n`;

            error_text += "\n# Ruffle Info\n";
            error_text += `Ruffle version: ${window.RufflePlayer.version}\n`;
            error_text += `Ruffle source: ${window.RufflePlayer.name}\n`;
            this.container.querySelector(
                "#panic-body"
            ).innerHTML = `<textarea>${error_text}</textarea>`;
            return false;
        };

        // Do this last, just in case it causes any cascading issues.
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
        }
    }

    debug_player_info() {
        return `Allows script access: ${this.allow_script_access}\n`;
    }
};

/*
 * Returns whether the given filename ends in an "swf" extension.
 */
exports.is_swf_filename = function is_swf_filename(filename) {
    return (
        filename &&
        typeof filename === "string" &&
        (filename.search(/\.swf(?:[?#]|$)/i) >= 0 ||
            filename.search(/\.spl(?:[?#]|$)/i) >= 0)
    );
};

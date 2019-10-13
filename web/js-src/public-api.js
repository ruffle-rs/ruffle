/**
 * Represents the Ruffle public API.
 * 
 * The public API exists primarily to allow multiple installs of Ruffle on a
 * page (e.g. an extension install and a local one) to cooperate. The first to
 * load "wins" and installs it's Public API class, and then all other Ruffle
 * sources install their sources into the Public API.
 * 
 * As a result, this class's functionality needs to be backwards and forwards
 * compatible, so it is very minimal. Any proposed change to the class must be
 * tested against previous self-hosted versions of Ruffle. We also allow target
 * pages to construct a minimally useful version of this class to declare
 * configuration before Ruffle has actually had a chance to load. Thus, this
 * class's constructor specifically allows a previous version of the Public API
 * to be upgraded from.
 */
export class PublicAPI {
    /**
     * Construct the Ruffle public API.
     * 
     * Do not use this function to negotiate a public API. Instead, use
     * `public_api` to install your Ruffle source into an existing public API
     * if it exists.
     * 
     * @param {object} prev What used to be in the public API slot.
     * 
     * This is used to upgrade from a prior version of the public API, or from
     * a user-defined configuration object placed in the public API slot.
     */
    constructor(prev) {
        this.sources = {};
        this.config = {};
        this.invoked = false;

        if (prev !== undefined) {
            if (prev.constructor.name === PublicAPI.name) {
                /// We're upgrading from a previous API to a new one.
                this.sources = prev.sources;
                this.config = prev.config;
                this.invoked = prev.invoked;
                this.conflict = prev.conflict;
            } else if (prev.constructor === Object && prev.interdictions !== undefined) {
                /// We're the first, install user configuration
                this.config = prev;
            } else {
                /// We're the first, but conflicting with someone else.
                this.conflict = prev;
            }
        }
    }

    /**
     * Install a given source into the Ruffle Public API.
     * 
     * @param {string} name The name of the source.
     * @param {object} api The public API object. This must conform to the shape
     * of `SourceAPI`.
     */
    install_source(name, api) {

    }

    /**
     * Negotiate and start Ruffle.
     * 
     * @param {array} interdictions The list of interdictions to configure.
     */
    init(interdictions) {
        window.RufflePlayer.invoked = true;
        
        for (var key in this.sources) {
            if (this.sources.hasOwnProperty(key)) {
                return this.sources[key].init(interdictions)
            }
        }
    }

    /**
     * Join a source into the public API, if it doesn't already exist.
     * 
     * @param {*} prev_ruffle The previous iteration of the Ruffle API.
     * 
     * The `prev_ruffle` param lists the previous object in the RufflePlayer
     * slot. We perform some checks to see if this is a Ruffle public API or a
     * conflicting object. If this is conflicting, then a new public API will
     * be constructed (see the constructor information for what happens to
     * `prev_ruffle`).
     * 
     * @param {string|undefined} source_name The name of this particular
     * Ruffle source.
     * 
     * @param {object|undefined} source_api The Ruffle source to add.
     * 
     * If both parameters are provided they will be used to define a new Ruffle
     * source to install into the public API.
     * 
     * @returns {object} The Ruffle Public API.
     */
    static negotiate(prev_ruffle, source_name, source_api) {
        let public_api;
        if (prev_ruffle !== undefined && prev_ruffle.constructor.name == PublicAPI.name) {
            public_api = prev_ruffle;
        } else {
            public_api = new PublicAPI(prev_ruffle);
        }
        
        if (source_name !== undefined && source_api !== undefined) {
            public_api.install_source(source_name, source_api);
        }

        return public_api;
    };
}
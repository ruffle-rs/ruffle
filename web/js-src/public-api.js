import { Version } from "./version.js";

/**
 * Represents the Ruffle public API.
 * 
 * The public API exists primarily to allow multiple installs of Ruffle on a
 * page (e.g. an extension install and a local one) to cooperate. The first to
 * load "wins" and installs it's Public API class, and then all other Ruffle
 * sources register themselves with the Public API.
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
     * `public_api` to register your Ruffle source with an existing public API
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
        this.newest_name = false;

        if (prev !== undefined) {
            if (prev.constructor.name === PublicAPI.name) {
                /// We're upgrading from a previous API to a new one.
                this.sources = prev.sources;
                this.config = prev.config;
                this.invoked = prev.invoked;
                this.conflict = prev.conflict;
                this.newest_name = prev.newest_name;
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
     * Register a given source with the Ruffle Public API.
     * 
     * @param {string} name The name of the source.
     * @param {object} api The public API object. This must conform to the shape
     * of `SourceAPI`.
     */
    register_source(name, api) {
        this.sources[name] = api;
        this.newest_name = this.newest_source_name();
    }
    
    /**
     * Determine the name of the newest registered source in the Public API.
     * 
     * @returns {(string|bool)} The name of the source, or `false` if no source
     * has yet to be registered.
     */
    newest_source_name() {
        let newest_name = false, newest_version = Version.from_semver("0.0.0");

        for (let k in this.sources) {
            if (this.sources.hasOwnProperty(k)) {
                let k_version = Version.from_semver(this.sources[k].version);
                if (k_version.has_precedence_over(newest_version)) {
                    newest_name = k;
                    newest_version = k_version;
                }
            }
        }

        return newest_name;
    }

    /**
     * Negotiate and start Ruffle.
     * 
     * @param {array} interdictions The list of interdictions to configure.
     */
    init(interdictions) {
        window.RufflePlayer.invoked = true;

        if (this.newest_name === false) {
            throw new Error("No registered Ruffle source!");
        }
        
        this.sources[this.newest_name].init(interdictions);
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
     * source to register with the public API.
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
            public_api.register_source(source_name, source_api);
        }

        return public_api;
    };
}
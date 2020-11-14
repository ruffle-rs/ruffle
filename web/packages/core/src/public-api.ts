import { Version } from "./version";
import { VersionRange } from "./version-range";
import { SourceAPI } from "./source-api";

/**
 * Represents the Ruffle public API.
 *
 * The public API exists primarily to allow multiple installs of Ruffle on a
 * page (e.g. an extension install and a local one) to cooperate. In an ideal
 * situation, all Ruffle sources on the page install themselves into a single
 * public API, and then the public API picks the newest version by default.
 *
 * This API *is* versioned, in case we need to upgrade it. However, it must be
 * backwards- and forwards-compatible with all known sources.
 */
export class PublicAPI {
    private sources: Record<string, SourceAPI>;
    private config: any;
    private invoked: boolean;
    private newest_name: string | null;
    private conflict: any;

    /**
     * Construct the Ruffle public API.
     *
     * Do not use this function to negotiate a public API. Instead, use
     * `public_api` to register your Ruffle source with an existing public API
     * if it exists.
     *
     * Constructing a Public API will also trigger it to initialize Ruffle once
     * the page loads, if the API has not already been superseded.
     *
     * @param {object} prev What used to be in the public API slot.
     *
     * This is used to upgrade from a prior version of the public API, or from
     * a user-defined configuration object placed in the public API slot.
     */
    constructor(prev: any) {
        this.sources = {};
        this.config = {};
        this.invoked = false;
        this.newest_name = null;

        if (prev !== undefined && prev !== null) {
            if (prev.constructor.name === PublicAPI.name) {
                /// We're upgrading from a previous API to a new one.
                this.sources = prev.sources;
                this.config = prev.config;
                this.invoked = prev.invoked;
                this.conflict = prev.conflict;
                this.newest_name = prev.newest_name;

                prev.superseded();
            } else if (
                prev.constructor === Object &&
                prev.config !== undefined
            ) {
                /// We're the first, install user configuration
                this.config = prev.config;
            } else {
                /// We're the first, but conflicting with someone else.
                this.conflict = prev;
            }
        }

        if (document.readyState === "loading") {
            window.addEventListener("DOMContentLoaded", this.init.bind(this));
        } else {
            window.setTimeout(this.init.bind(this), 0);
        }
    }

    /**
     * The version of the public API.
     *
     * This allows a page with an old version of the Public API to be upgraded
     * to a new version of the API. The public API is intended to be changed
     * very infrequently, if at all, but this provides an escape mechanism for
     * newer Ruffle sources to upgrade older installations.
     */
    get version() {
        return "0.1.0";
    }

    /**
     * Register a given source with the Ruffle Public API.
     *
     * @param {string} name The name of the source.
     * @param {object} api The public API object. This must conform to the shape
     * of `SourceAPI`.
     */
    register_source(name: string, api: SourceAPI) {
        this.sources[name] = api;
    }

    /**
     * Determine the name of the newest registered source in the Public API.
     *
     * @returns {(string|bool)} The name of the source, or `false` if no source
     * has yet to be registered.
     */
    newest_source_name() {
        let newest_name = null,
            newest_version = Version.fromSemver("0.0.0");

        for (const k in this.sources) {
            if (Object.prototype.hasOwnProperty.call(this.sources, k)) {
                const k_version = Version.fromSemver(this.sources[k].version);
                if (k_version.hasPrecedenceOver(newest_version)) {
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
     * This function reads the config parameter to determine which polyfills
     * should be enabled. If the configuration parameter is missing, then we
     * use a built-in set of defaults sufficient to fool sites with static
     * content and weak plugin detection.
     */
    init() {
        if (!this.invoked) {
            this.invoked = true;
            this.newest_name = this.newest_source_name();

            if (this.newest_name === null) {
                throw new Error("No registered Ruffle source!");
            }

            const polyfills = this.config.polyfills;
            if (polyfills !== false) {
                this.sources[this.newest_name].polyfill();
            }
        }
    }

    /**
     * Look up the newest Ruffle source and return it's API.
     *
     * @returns {SourceAPI} An instance of the Source API.
     */
    newest() {
        const name = this.newest_source_name();
        return name != null ? this.sources[name] : null;
    }

    /**
     * Look up a specific Ruffle version (or any version satisfying a given set
     * of requirements) and return it's API.
     *
     * @param {string} ver_requirement A set of semantic version requirement
     * strings that the player version must satisfy.
     *
     * @returns {SourceAPI|null} An instance of the Source API, if one or more
     * sources satisfied the requirement.
     */
    satisfying(ver_requirement: string) {
        const requirement = VersionRange.fromRequirementString(ver_requirement);
        let valid_source = null;

        for (const k in this.sources) {
            if (Object.prototype.hasOwnProperty.call(this.sources, k)) {
                const version = Version.fromSemver(this.sources[k].version);

                if (requirement.satisfiedBy(version)) {
                    valid_source = this.sources[k];
                }
            }
        }

        return valid_source;
    }

    /**
     * Look up the newest Ruffle version compatible with the `local` source, if
     * it's installed. Otherwise, use the latest version.
     */
    local_compatible() {
        if (this.sources.local !== undefined) {
            return this.satisfying("^" + this.sources.local.version);
        } else {
            return this.newest();
        }
    }

    /**
     * Look up the newest Ruffle version with the exact same version as the
     * `local` source, if it's installed. Otherwise, use the latest version.
     */
    local() {
        if (this.sources.local !== undefined) {
            return this.satisfying("=" + this.sources.local.version);
        } else {
            return this.newest();
        }
    }

    /**
     * Indicates that this version of the public API has been superseded by a
     * newer version.
     *
     * This should only be called by a newer version of the Public API.
     * Identical versions of the Public API should not supersede older versions
     * of that same API.
     *
     * Unfortunately, we can't disable polyfills after-the-fact, so this
     * only lets you disable the init event...
     */
    superseded() {
        this.invoked = true;
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
     * Note that Public API upgrades are deliberately not enabled in this
     * version of Ruffle, since there is no Public API to upgrade from.
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
    static negotiate(
        prev_ruffle: any,
        source_name: string | undefined,
        source_api: SourceAPI | undefined
    ) {
        let public_api;
        if (
            prev_ruffle !== undefined &&
            prev_ruffle.constructor.name == PublicAPI.name
        ) {
            public_api = prev_ruffle;
        } else {
            public_api = new PublicAPI(prev_ruffle);
        }

        if (source_name !== undefined && source_api !== undefined) {
            public_api.register_source(source_name, source_api);

            // Install the faux plugin detection immediately.
            // This is necessary because scripts such as SWFObject check for the
            // Flash Player immediately when they load.
            // TODO: Maybe there's a better place for this.
            const polyfills = public_api.config.polyfills;
            if (polyfills !== false) {
                source_api.pluginPolyfill();
            }
        }

        return public_api;
    }
}

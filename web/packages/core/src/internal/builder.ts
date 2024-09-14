import type { RuffleInstanceBuilder } from "../../dist/ruffle_web";
import { BaseLoadOptions, Duration, SecsDuration } from "../load-options";

/**
 * Checks if the given value is explicitly `T` (not null, not undefined)
 *
 * @param value The value to test
 * @returns true if the value isn't null or undefined
 */
function isExplicit<T>(value: T | undefined | null): value is T {
    return value !== null && value !== undefined;
}

/**
 * Configures the given RuffleInstanceBuilder for the general options provided.
 *
 * This is the translation layer between what we allow users to provide through e.g. `window.RufflePlayer.config`,
 * which is quite relaxed and may evolve over time,
 * and the actual values we accept inside Rust (which is quite strict).
 *
 * This allows us to change the rust side at will, and without needing to worry about backwards compatibility, parsing, etc.
 *
 * @param builder The builder to set the options on
 * @param config The options to apply
 */
export function configureBuilder(
    builder: RuffleInstanceBuilder,
    config: BaseLoadOptions,
) {
    // Guard things for being explicitly set, so that we don't need to specify defaults in yet another place...

    if (isExplicit(config.allowScriptAccess)) {
        builder.setAllowScriptAccess(config.allowScriptAccess);
    }
    if (isExplicit(config.backgroundColor)) {
        builder.setBackgroundColor(parseColor(config.backgroundColor));
    }
    if (isExplicit(config.upgradeToHttps)) {
        builder.setUpgradeToHttps(config.upgradeToHttps);
    }
    if (isExplicit(config.compatibilityRules)) {
        builder.setCompatibilityRules(config.compatibilityRules);
    }
    if (isExplicit(config.letterbox)) {
        builder.setLetterbox(config.letterbox.toLowerCase());
    }
    if (isExplicit(config.base)) {
        builder.setBaseUrl(config.base);
    }
    if (isExplicit(config.menu)) {
        builder.setShowMenu(config.menu);
    }
    if (isExplicit(config.allowFullscreen)) {
        builder.setAllowFullscreen(config.allowFullscreen);
    }
    if (isExplicit(config.salign)) {
        builder.setStageAlign(config.salign.toLowerCase());
    }
    if (isExplicit(config.forceAlign)) {
        builder.setForceAlign(config.forceAlign);
    }
    if (isExplicit(config.quality)) {
        builder.setQuality(config.quality.toLowerCase());
    } else if (isMobileOrTablet()) {
        console.log("Running on a mobile device; defaulting to low quality");
        builder.setQuality("low");
    }
    if (isExplicit(config.scale)) {
        builder.setScale(config.scale.toLowerCase());
    }
    if (isExplicit(config.forceScale)) {
        builder.setForceScale(config.forceScale);
    }
    if (isExplicit(config.frameRate)) {
        builder.setFrameRate(config.frameRate);
    }
    if (isExplicit(config.wmode)) {
        builder.setWmode(config.wmode);
    }
    if (isExplicit(config.logLevel)) {
        builder.setLogLevel(config.logLevel);
    }
    if (isExplicit(config.maxExecutionDuration)) {
        builder.setMaxExecutionDuration(
            parseDuration(config.maxExecutionDuration),
        );
    }
    if (isExplicit(config.playerVersion)) {
        builder.setPlayerVersion(config.playerVersion);
    }
    if (isExplicit(config.preferredRenderer)) {
        builder.setPreferredRenderer(config.preferredRenderer);
    }
    if (isExplicit(config.openUrlMode)) {
        builder.setOpenUrlMode(config.openUrlMode.toLowerCase());
    }
    if (isExplicit(config.allowNetworking)) {
        builder.setAllowNetworking(config.allowNetworking.toLowerCase());
    }
    if (isExplicit(config.credentialAllowList)) {
        builder.setCredentialAllowList(config.credentialAllowList);
    }
    if (isExplicit(config.playerRuntime)) {
        builder.setPlayerRuntime(config.playerRuntime);
    }

    if (isExplicit(config.socketProxy)) {
        for (const proxy of config.socketProxy) {
            builder.addSocketProxy(proxy.host, proxy.port, proxy.proxyUrl);
        }
    }
}

/**
 * Parses a color into an RGB value.
 *
 * @param color The color string to parse
 * @returns A valid RGB number, or undefined if invalid
 */
export function parseColor(color: string): number | undefined {
    if (color.startsWith("#")) {
        color = color.substring(1);
    }
    if (color.length < 6) {
        return undefined;
    }
    let result = 0;

    for (let i = 0; i < 6; i++) {
        const digit = parseInt(color[i]!, 16);
        if (!isNaN(digit)) {
            result = (result << 4) | digit;
        } else {
            result = result << 4;
        }
    }

    return result;
}

/**
 * Parses a duration into number of seconds.
 *
 * @param value The duration to parse
 * @returns A valid number of seconds
 */
export function parseDuration(value: Duration): SecsDuration {
    if (typeof value === "number") {
        return value;
    }
    return value.secs;
}

/**
 * Very bad way to guess if we're running on a tablet/mobile.
 *
 * @returns True if we believe this may be a mobile or tablet device
 */
function isMobileOrTablet(): boolean {
    // noinspection JSDeprecatedSymbols
    return typeof window.orientation !== "undefined";
}

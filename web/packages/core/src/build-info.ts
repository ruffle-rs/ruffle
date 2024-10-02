/**
 * Stores build information about <b>this specific version of the `ruffle-core` library</b>.
 *
 * It does not represent the version of Ruffle that may be in use by the page (for example, if a browser extension overrides it).
 *
 * @privateRemarks
 * This is generated at build time via `set_version.ts`.
 */
export const buildInfo = {
    versionNumber: "%VERSION_NUMBER%",
    versionName: "%VERSION_NAME%",
    versionChannel: "%VERSION_CHANNEL%",
    buildDate: "%BUILD_DATE%",
    commitHash: "%COMMIT_HASH%",
};

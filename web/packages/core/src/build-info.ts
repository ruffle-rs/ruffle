/**
 * Stores build information. The string literals are replaces at compile time by `set_version.js`.
 */
export const buildInfo = {
    versionNumber: "%VERSION_NUMBER%",
    versionName: "%VERSION_NAME%",
    versionChannel: "%VERSION_CHANNEL%",
    buildDate: "%BUILD_DATE%",
    commitHash: "%COMMIT_HASH%",
};

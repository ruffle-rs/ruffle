/**
 * Stores build information. The string literals are replaced at compile time by `set_version.ts`.
 */
export const buildInfo = {
    versionNumber: "%VERSION_NUMBER%",
    versionName: "%VERSION_NAME%",
    versionChannel: "%VERSION_CHANNEL%",
    buildDate: "%BUILD_DATE%",
    commitHash: "%COMMIT_HASH%",
};

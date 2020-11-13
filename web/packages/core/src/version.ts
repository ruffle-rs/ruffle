/**
 * A representation of a semver 2 compliant version string
 */
export class Version {
    private readonly major: number;
    private readonly minor: number;
    private readonly patch: number;
    private readonly prIdent: string[] | null;
    private readonly buildIdent: string[] | null;

    /**
     * Construct a Version from specific components.
     *
     * If you wish to parse a string into a Version then please use [[from_semver]].
     *
     * @param major The major version component.
     * @param minor The minor version component.
     * @param patch The patch version component.
     * @param pr_ident A list of pre-release identifiers, if any
     * @param build_ident A list of build identifiers, if any
     */
    constructor(
        major: number,
        minor: number,
        patch: number,
        pr_ident: string[] | null,
        build_ident: string[] | null
    ) {
        this.major = major;
        this.minor = minor;
        this.patch = patch;
        this.prIdent = pr_ident;
        this.buildIdent = build_ident;
    }

    /**
     * Construct a version from a semver 2 compliant string.
     *
     * This function is intended for use with semver 2 compliant strings.
     * Malformed strings may still parse correctly, but this result is not
     * guaranteed.
     *
     * @param version_string A semver 2.0.0 compliant version string
     * @return A version object
     */
    static from_semver(version_string: string): Version {
        const build_split = version_string.split("+"),
            pr_split = build_split[0].split("-"),
            version_split = pr_split[0].split(".");

        const major = parseInt(version_split[0], 10);
        let minor = 0;
        let patch = 0;
        let pr_ident = null;
        let build_ident = null;

        if (version_split[1] != undefined) {
            minor = parseInt(version_split[1], 10);
        }

        if (version_split[2] != undefined) {
            patch = parseInt(version_split[2], 10);
        }

        if (pr_split[1] != undefined) {
            pr_ident = pr_split[1].split(".");
        }

        if (build_split[1] != undefined) {
            build_ident = build_split[1].split(".");
        }

        return new Version(major, minor, patch, pr_ident, build_ident);
    }

    /**
     * Returns true if a given version is compatible with this one.
     *
     * Compatibility is defined as having the same nonzero major version
     * number, or if both major versions are zero, the same nonzero minor
     * version number, or if both minor versions are zero, the same nonzero
     * patch version number.
     *
     * This implements the ^ operator in npm's semver package, with the
     * exception of the prerelease exclusion rule.
     *
     * @param fver The other version to test against
     * @return True if compatible
     */
    is_compatible_with(fver: Version): boolean {
        return (
            (this.major !== 0 && this.major === fver.major) ||
            (this.major === 0 &&
                fver.major === 0 &&
                this.minor !== 0 &&
                this.minor === fver.minor) ||
            (this.major === 0 &&
                fver.major === 0 &&
                this.minor === 0 &&
                fver.minor === 0 &&
                this.patch !== 0 &&
                this.patch === fver.patch)
        );
    }

    /**
     * Returns true if this version has precedence over (is newer than) another
     * version.
     *
     * Precedence is defined as in the Semver 2 spec. This implements the >
     * operator in npm's semver package, with the exception of the prerelease
     * exclusion rule.
     *
     * @param fver The other version to test against
     * @return True if this version has precedence over the other one
     */
    has_precedence_over(fver: Version): boolean {
        if (this.major > fver.major) {
            return true;
        } else if (this.major < fver.major) {
            return false;
        }

        if (this.minor > fver.minor) {
            return true;
        } else if (this.minor < fver.minor) {
            return false;
        }

        if (this.patch > fver.patch) {
            return true;
        } else if (this.patch < fver.patch) {
            return false;
        }

        if (this.prIdent == null && fver.prIdent != null) {
            return true;
        } else if (this.prIdent != null && fver.prIdent != null) {
            const is_numeric = /^[0-9]*$/;
            for (
                let i = 0;
                i < this.prIdent.length && i < fver.prIdent.length;
                i += 1
            ) {
                if (
                    !is_numeric.test(this.prIdent[i]) &&
                    is_numeric.test(fver.prIdent[i])
                ) {
                    return true;
                } else if (
                    is_numeric.test(this.prIdent[i]) &&
                    is_numeric.test(fver.prIdent[i])
                ) {
                    if (
                        parseInt(this.prIdent[i], 10) >
                        parseInt(fver.prIdent[i], 10)
                    ) {
                        return true;
                    } else if (
                        parseInt(this.prIdent[i], 10) <
                        parseInt(fver.prIdent[i], 10)
                    ) {
                        return false;
                    }
                } else if (
                    is_numeric.test(this.prIdent[i]) &&
                    !is_numeric.test(fver.prIdent[i])
                ) {
                    return false;
                } else if (
                    !is_numeric.test(this.prIdent[i]) &&
                    !is_numeric.test(fver.prIdent[i])
                ) {
                    if (this.prIdent[i] > fver.prIdent[i]) {
                        return true;
                    } else if (this.prIdent[i] < fver.prIdent[i]) {
                        return false;
                    }
                }
            }

            return this.prIdent.length > fver.prIdent.length;
        }

        return false;
    }

    /**
     * Tests if a given version is equivalent to this one.
     *
     * Build and prerelease tags are ignored.
     *
     * @param fver The other version to test against
     * @return True if the given version is equivalent
     */
    is_equal(fver: Version): boolean {
        return (
            this.major === fver.major &&
            this.minor === fver.minor &&
            this.patch === fver.patch
        );
    }

    /**
     * Tests if a given version is stable or a compatible prerelease for this
     * version.
     *
     * This implements the prerelease exclusion rule of NPM semver: a
     * prerelease version can only pass this check if the major/minor/patch
     * components of both versions are the same. Otherwise, the prerelease
     * version always fails.
     *
     * @param fver The other version to test against
     * @return True if the given version is either stable, or a
     * prerelease in the same series as this one.
     */
    is_stable_or_compatible_prerelease(fver: Version): boolean {
        if (fver.prIdent == null) {
            return true;
        } else {
            return (
                this.major === fver.major &&
                this.minor === fver.minor &&
                this.patch === fver.patch
            );
        }
    }
}

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
     * If you wish to parse a string into a Version then please use [[fromSemver]].
     *
     * @param major The major version component.
     * @param minor The minor version component.
     * @param patch The patch version component.
     * @param prIdent A list of pre-release identifiers, if any
     * @param buildIdent A list of build identifiers, if any
     */
    constructor(
        major: number,
        minor: number,
        patch: number,
        prIdent: string[] | null,
        buildIdent: string[] | null
    ) {
        this.major = major;
        this.minor = minor;
        this.patch = patch;
        this.prIdent = prIdent;
        this.buildIdent = buildIdent;
    }

    /**
     * Construct a version from a semver 2 compliant string.
     *
     * This function is intended for use with semver 2 compliant strings.
     * Malformed strings may still parse correctly, but this result is not
     * guaranteed.
     *
     * @param versionString A semver 2.0.0 compliant version string
     * @returns A version object
     */
    static fromSemver(versionString: string): Version {
        const buildSplit = versionString.split("+"),
            prSplit = buildSplit[0].split("-"),
            versionSplit = prSplit[0].split(".");

        const major = parseInt(versionSplit[0], 10);
        let minor = 0;
        let patch = 0;
        let prIdent = null;
        let buildIdent = null;

        if (versionSplit[1] !== undefined) {
            minor = parseInt(versionSplit[1], 10);
        }

        if (versionSplit[2] !== undefined) {
            patch = parseInt(versionSplit[2], 10);
        }

        if (prSplit[1] !== undefined) {
            prIdent = prSplit[1].split(".");
        }

        if (buildSplit[1] !== undefined) {
            buildIdent = buildSplit[1].split(".");
        }

        return new Version(major, minor, patch, prIdent, buildIdent);
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
     * @param other The other version to test against
     * @returns True if compatible
     */
    isCompatibleWith(other: Version): boolean {
        return (
            (this.major !== 0 && this.major === other.major) ||
            (this.major === 0 &&
                other.major === 0 &&
                this.minor !== 0 &&
                this.minor === other.minor) ||
            (this.major === 0 &&
                other.major === 0 &&
                this.minor === 0 &&
                other.minor === 0 &&
                this.patch !== 0 &&
                this.patch === other.patch)
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
     * @param other The other version to test against
     * @returns True if this version has precedence over the other one
     */
    hasPrecedenceOver(other: Version): boolean {
        if (this.major > other.major) {
            return true;
        } else if (this.major < other.major) {
            return false;
        }

        if (this.minor > other.minor) {
            return true;
        } else if (this.minor < other.minor) {
            return false;
        }

        if (this.patch > other.patch) {
            return true;
        } else if (this.patch < other.patch) {
            return false;
        }

        if (this.prIdent === null && other.prIdent !== null) {
            return true;
        } else if (this.prIdent !== null && other.prIdent !== null) {
            const isNumeric = /^[0-9]*$/;
            for (
                let i = 0;
                i < this.prIdent.length && i < other.prIdent.length;
                i += 1
            ) {
                if (
                    !isNumeric.test(this.prIdent[i]) &&
                    isNumeric.test(other.prIdent[i])
                ) {
                    return true;
                } else if (
                    isNumeric.test(this.prIdent[i]) &&
                    isNumeric.test(other.prIdent[i])
                ) {
                    if (
                        parseInt(this.prIdent[i], 10) >
                        parseInt(other.prIdent[i], 10)
                    ) {
                        return true;
                    } else if (
                        parseInt(this.prIdent[i], 10) <
                        parseInt(other.prIdent[i], 10)
                    ) {
                        return false;
                    }
                } else if (
                    isNumeric.test(this.prIdent[i]) &&
                    !isNumeric.test(other.prIdent[i])
                ) {
                    return false;
                } else if (
                    !isNumeric.test(this.prIdent[i]) &&
                    !isNumeric.test(other.prIdent[i])
                ) {
                    if (this.prIdent[i] > other.prIdent[i]) {
                        return true;
                    } else if (this.prIdent[i] < other.prIdent[i]) {
                        return false;
                    }
                }
            }

            return this.prIdent.length > other.prIdent.length;
        }

        return false;
    }

    /**
     * Tests if a given version is equivalent to this one.
     *
     * Build and prerelease tags are ignored.
     *
     * @param other The other version to test against
     * @returns True if the given version is equivalent
     */
    isEqual(other: Version): boolean {
        return (
            this.major === other.major &&
            this.minor === other.minor &&
            this.patch === other.patch
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
     * @param other The other version to test against
     * @returns True if the given version is either stable, or a
     * prerelease in the same series as this one.
     */
    isStableOrCompatiblePrerelease(other: Version): boolean {
        if (other.prIdent === null) {
            return true;
        } else {
            return (
                this.major === other.major &&
                this.minor === other.minor &&
                this.patch === other.patch
            );
        }
    }
}

export class Version {
    /**
     * Construct a Version from components.
     * 
     * @param {number} major The major version
     * @param {number} minor The minor version
     * @param {number} patch The patch version
     * @param {array|undefined} pr_ident A list of pre-release identifiers, if
     * any.
     * @param {array|undefined} build_ident A list of build identifiers, if
     * any.
     */
    constructor(major, minor, patch, pr_ident, build_ident) {
        this.major = major;
        this.minor = minor;
        this.patch = patch;
        this.pr_ident = pr_ident;
        this.build_ident = build_ident;
    }

    /**
     * Construct a version from a semver 2 compliant string.
     * 
     * This function is intended for use with semver 2 compliant strings.
     * Malformatted strins may still parse correctly, however.
     * 
     * @param {string} version_string A semver 2.0.0 compliant version string.
     * @return {Version} A version object.
     */
    static from_semver(version_string) {
        let build_split = version_string.split("+"),
            pr_split = build_split[0].split("-"),
            version_split = pr_split[0].split("."),
            version = [];
        
        version.push(parseInt(version_split[0]));

        if (version_split[1] != undefined) {
            version.push(parseInt(version_split[1]));
        } else {
            version.push(0);
        }

        if (version_split[2] != undefined) {
            version.push(parseInt(version_split[2]));
        } else {
            version.push(0);
        }

        if (pr_split[1] != undefined) {
            version.push(pr_split[1].split("."));
        } else {
            version.push(undefined);
        }

        if (build_split[1] != undefined) {
            version.push(build_split[1].split("."));
        } else {
            version.push(undefined);
        }

        return new Version(version[0], version[1], version[2], version[3], version[4]);
    }

    /**
     * Returns true if a given version is compatible with this one.
     * 
     * Compatibility is defined as having the same nonzero major version
     * number, or if both major versions are zero, the same nonzero minor
     * version number, or if both minor versions are zero, the same nonzero
     * patch version number.
     * 
     * @param {Version} fver The other version to test against
     * @return {bool}
     */
    is_compatible_with(fver) {
        return this.major !== 0 && this.major === fver.major ||
            this.major === 0 && fver.major === 0 && this.minor === fver.minor ||
            this.major === 0 && fver.major === 0 && this.minor === 0 && fver.minor === 0 && this.patch === fver.patch;
    }

    /**
     * Returns true if this version has precedence over (is newer than) another
     * version.
     * 
     * Precedence is defined as in the Semver 2 spec.
     * 
     * @param {Version} fver The other version to test against
     * @return {bool} True if this version has precedence over the other one.
     */
    has_precedence_over(fver) {
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

        if (this.pr_ident === undefined && fver.pr_ident !== undefined) {
            return true;
        } else if (this.pr_ident !== undefined && fver.pr_ident !== undefined) {
            let is_numeric = /^[0-9]*$/;
            for (let i = 0; i < this.pr_ident.length && i < fver.pr_ident.length; i += 1) {
                if (!is_numeric.test(this.pr_ident[i]) && is_numeric.test(fver.pr_ident[i])) {
                    return true;
                } else if (is_numeric.test(this.pr_ident[i]) && is_numeric.test(fver.pr_ident[i])) {
                    if (parseInt(this.pr_ident[i]) > parseInt(fver.pr_ident[i])) {
                        return true;
                    } else if (parseInt(this.pr_ident[i]) > parseInt(fver.pr_ident[i])) {
                        return false;
                    }
                } else if (!is_numeric.test(this.pr_ident[i]) && is_numeric.test(fver.pr_ident[i])) {
                    return true;
                } else if (!is_numeric.test(this.pr_ident[i]) && !is_numeric.test(fver.pr_ident[i])) {
                    if (this.pr_ident[i] > fver.pr_ident[i]) {
                        return true;
                    } else if (this.pr_ident[i] > fver.pr_ident[i]) {
                        return false;
                    }
                }
            }
            
            return this.pr_ident.length > fver.pr_ident.length;
        }

        return false;
    }
}
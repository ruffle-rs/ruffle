const { Version } = require("./version");

/**
 * Represents a set of version requirements.
 */
exports.VersionRange = class VersionRange {
    constructor(requirements) {
        this.requirements = requirements;
    }

    /**
     * Determine if a given version satisfies this range.
     *
     * @param {Version} fver A version object to test against.
     * @return {bool} Whether or not the given version matches this range.
     */
    satisfied_by(fver) {
        for (let i = 0; i < this.requirements.length; i += 1) {
            let matches = true;

            for (let j = 0; j < this.requirements[i].length; j += 1) {
                let comparator = this.requirements[i][j][0];
                let version = this.requirements[i][j][1];

                matches =
                    matches && version.is_stable_or_compatible_prerelease(fver);

                if (comparator === "" || comparator === "=") {
                    matches = matches && version.is_equal(fver);
                } else if (comparator === ">") {
                    matches = matches && fver.has_precedence_over(version);
                } else if (comparator === ">=") {
                    matches =
                        matches &&
                        (fver.has_precedence_over(version) ||
                            version.is_equal(fver));
                } else if (comparator === "<") {
                    matches = matches && version.has_precedence_over(fver);
                } else if (comparator === "<=") {
                    matches =
                        matches &&
                        (version.has_precedence_over(fver) ||
                            version.is_equal(fver));
                } else if (comparator === "^") {
                    matches = matches && version.is_compatible_with(fver);
                }
            }

            if (matches) {
                return true;
            }
        }

        return false;
    }

    /**
     * Parse a requirement string into a version range.
     *
     * @param {string} requirement The version requirements, consisting of a
     * series of space-separated strings, each one being a semver version
     * optionally prefixed by a comparator (e.g. <, <=, >, >=, =, or ^), or the
     * string ||.
     * @return {VersionRange} A version range object.
     */
    static from_requirement_string(requirement) {
        let components = requirement.split(" ");
        let requirement_set = [];
        let requirements = [];

        for (let i = 0; i < components.length; i += 1) {
            if (components[i] === "||") {
                if (requirement_set.length > 0) {
                    requirements.push(requirement_set);
                    requirement_set = [];
                }
            } else if (components[i].length > 0) {
                let match = /[0-9]/.exec(components[i]);
                let comparator = components[i].slice(0, match.index).trim();
                let version = Version.from_semver(
                    components[i].slice(match.index).trim()
                );

                requirement_set.push([comparator, version]);
            }
        }

        if (requirement_set.length > 0) {
            requirements.push(requirement_set);
        }

        return new VersionRange(requirements);
    }
};

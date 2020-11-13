import { Version } from "./version";

interface Requirement {
    comparator: string;
    version: Version;
}

/**
 * Represents a set of version requirements.
 */
export class VersionRange {
    readonly requirements: Requirement[][];

    constructor(requirements: Requirement[][]) {
        this.requirements = requirements;
    }

    /**
     * Determine if a given version satisfies this range.
     *
     * @param {Version} fver A version object to test against.
     * @return {bool} Whether or not the given version matches this range.
     */
    satisfied_by(fver: Version) {
        for (let i = 0; i < this.requirements.length; i += 1) {
            let matches = true;

            for (let j = 0; j < this.requirements[i].length; j += 1) {
                const comparator = this.requirements[i][j].comparator;
                const version = this.requirements[i][j].version;

                matches =
                    matches && version.is_stable_or_compatible_prerelease(fver);

                if (comparator === "" || comparator === "=") {
                    matches = matches && version.isEqual(fver);
                } else if (comparator === ">") {
                    matches = matches && fver.hasPrecedenceOver(version);
                } else if (comparator === ">=") {
                    matches =
                        matches &&
                        (fver.hasPrecedenceOver(version) ||
                            version.isEqual(fver));
                } else if (comparator === "<") {
                    matches = matches && version.hasPrecedenceOver(fver);
                } else if (comparator === "<=") {
                    matches =
                        matches &&
                        (version.hasPrecedenceOver(fver) ||
                            version.isEqual(fver));
                } else if (comparator === "^") {
                    matches = matches && version.isCompatibleWith(fver);
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
    static from_requirement_string(requirement: string) {
        const components = requirement.split(" ");
        let requirement_set: Requirement[] = [];
        const requirements: Requirement[][] = [];

        for (let i = 0; i < components.length; i += 1) {
            if (components[i] === "||") {
                if (requirement_set.length > 0) {
                    requirements.push(requirement_set);
                    requirement_set = [];
                }
            } else if (components[i].length > 0) {
                const match = /[0-9]/.exec(components[i]);
                if (match) {
                    const comparator = components[i]
                        .slice(0, match.index)
                        .trim();
                    const version = Version.fromSemver(
                        components[i].slice(match.index).trim()
                    );

                    requirement_set.push({ comparator, version });
                }
            }
        }

        if (requirement_set.length > 0) {
            requirements.push(requirement_set);
        }

        return new VersionRange(requirements);
    }
}

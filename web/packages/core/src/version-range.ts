import { Version } from "./version";

/**
 * A requirement is a comparator (such as ">" or "=" or "")
 * and a version to compare against.
 */
interface Requirement {
    /**
     * A comparator is the operation to use with this requirement.
     *
     * Valid options are as follows:
     *
     * - `""` or `"="`: Precisely this version
     * - `">`": A version newer than this one
     * - `">`=": A version newer or equal to this one
     * - `"<"`: A version older than this one
     * - `"<="`: A version older or equal to this one
     * - `"^"`: A version that is compatible with this one
     */
    comparator: string;

    /**
     * The version to perform the test against.
     */
    version: Version;
}

/**
 * Represents a set of version requirements.
 */
export class VersionRange {
    /**
     * The list of requirements used by this version range.
     *
     * This is a disjunctive normal form - that is, an OR of ANDs.
     *
     * If all requirements of a single inner array match, the range is
     * considered successful.
     */
    readonly requirements: Requirement[][];

    /**
     * Constructs a range of versions as specified by the given requirements.
     *
     * If you wish to construct this object from a string representation,
     * then use [[fromRequirementString]].
     *
     * @param requirements Requirements to set this range by
     */
    constructor(requirements: Requirement[][]) {
        this.requirements = requirements;
    }

    /**
     * Determine if a given version satisfies this range.
     *
     * @param fver A version object to test against.
     * @returns Whether or not the given version matches this range
     */
    satisfiedBy(fver: Version): boolean {
        for (let i = 0; i < this.requirements.length; i += 1) {
            let matches = true;

            for (let j = 0; j < this.requirements[i].length; j += 1) {
                const comparator = this.requirements[i][j].comparator;
                const version = this.requirements[i][j].version;

                matches =
                    matches && version.isStableOrCompatiblePrerelease(fver);

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
     * @param requirement The version requirements, consisting of a
     * series of space-separated strings, each one being a semver version
     * optionally prefixed by a comparator or a separator.
     *
     * Valid comparators are:
     * - `""` or `"="`: Precisely this version
     * - `">`": A version newer than this one
     * - `">`=": A version newer or equal to this one
     * - `"<"`: A version older than this one
     * - `"<="`: A version older or equal to this one
     * - `"^"`: A version that is compatible with this one
     *
     * A separator is `"||`" which splits the requirement string into
     * left OR right.
     * @returns A version range object.
     */
    static fromRequirementString(requirement: string): VersionRange {
        const components = requirement.split(" ");
        let set: Requirement[] = [];
        const requirements: Requirement[][] = [];

        for (let i = 0; i < components.length; i += 1) {
            if (components[i] === "||") {
                if (set.length > 0) {
                    requirements.push(set);
                    set = [];
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

                    set.push({ comparator, version });
                }
            }
        }

        if (set.length > 0) {
            requirements.push(set);
        }

        return new VersionRange(requirements);
    }
}

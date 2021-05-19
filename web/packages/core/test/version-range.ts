import { strict as assert } from "assert";
import { VersionRange } from "../src/version-range";
import { Version } from "../src/version";

describe("VersionRange", function () {
    describe("#from_requirement_string()", function () {
        it("should accept a specific version without an equals sign", function () {
            const range = VersionRange.fromRequirementString("1.2.3");
            assert.deepEqual(range.requirements, [
                [{ comparator: "", version: Version.fromSemver("1.2.3") }],
            ]);
        });

        it("should accept two different versions without equals signs", function () {
            const range = VersionRange.fromRequirementString("1.2.3 || 1.2.4");
            assert.deepEqual(range.requirements, [
                [{ comparator: "", version: Version.fromSemver("1.2.3") }],
                [{ comparator: "", version: Version.fromSemver("1.2.4") }],
            ]);
        });

        it("should accept a specific version with an equals sign", function () {
            const range = VersionRange.fromRequirementString("=1.2.3");
            assert.deepEqual(range.requirements, [
                [{ comparator: "=", version: Version.fromSemver("1.2.3") }],
            ]);
        });

        it("should accept a specific version with an equals sign", function () {
            const range =
                VersionRange.fromRequirementString("=1.2.3 || =1.2.4");
            assert.deepEqual(range.requirements, [
                [{ comparator: "=", version: Version.fromSemver("1.2.3") }],
                [{ comparator: "=", version: Version.fromSemver("1.2.4") }],
            ]);
        });

        it("should accept a min and max range", function () {
            const range = VersionRange.fromRequirementString(">1.2.3 <1.2.5");
            assert.deepEqual(range.requirements, [
                [
                    { comparator: ">", version: Version.fromSemver("1.2.3") },
                    { comparator: "<", version: Version.fromSemver("1.2.5") },
                ],
            ]);
        });

        it("should allow inclusive range", function () {
            const range =
                VersionRange.fromRequirementString(">=1-test <=2-test");
            assert.deepEqual(range.requirements, [
                [
                    {
                        comparator: ">=",
                        version: Version.fromSemver("1-test"),
                    },
                    {
                        comparator: "<=",
                        version: Version.fromSemver("2-test"),
                    },
                ],
            ]);
        });

        it("should ignore extra whitespace within a range", function () {
            const range = VersionRange.fromRequirementString("^1.2   <1.3");
            assert.deepEqual(range.requirements, [
                [
                    { comparator: "^", version: Version.fromSemver("1.2") },
                    { comparator: "<", version: Version.fromSemver("1.3") },
                ],
            ]);
        });

        it("should ignore empty ranges", function () {
            const range = VersionRange.fromRequirementString(
                "|| || 1.2.4 || || 1.2.5 ||"
            );
            assert.deepEqual(range.requirements, [
                [{ comparator: "", version: Version.fromSemver("1.2.4") }],
                [{ comparator: "", version: Version.fromSemver("1.2.5") }],
            ]);
        });
    });

    describe("#satisfied_by()", function () {
        const groups = [
            {
                requirements: "1.2.3",
                tests: [
                    { version: "1.2.3", expected: true },
                    { version: "1.2.4", expected: false },
                    { version: "1.2.2", expected: false },
                    { version: "1.2.3-test", expected: true },
                ],
            },
            {
                requirements: "1.2.3 || 1.2.4",
                tests: [
                    { version: "1.2.3", expected: true },
                    { version: "1.2.4", expected: true },
                    { version: "1.2.2", expected: false },
                    { version: "1.2.3-test", expected: true },
                    { version: "1.2.4+build", expected: true },
                ],
            },
            {
                requirements: "^1.2",
                tests: [
                    { version: "1.2", expected: true },
                    { version: "1.2.5", expected: true },
                    { version: "1.2.6-pre", expected: false },
                    { version: "1.3", expected: true },
                    { version: "2.0", expected: false },
                ],
            },
            {
                requirements: ">=1.2.3 <=1.3.2",
                tests: [
                    { version: "1.2", expected: false },
                    { version: "1.2.3", expected: true },
                    { version: "1.2.5", expected: true },
                    { version: "1.2.6+build", expected: true },
                    { version: "1.3.2", expected: true },
                    { version: "1.3.3", expected: false },
                ],
            },
            {
                requirements: ">1.2.3 <1.3.2",
                tests: [
                    { version: "1.2", expected: false },
                    { version: "1.2.3", expected: false },
                    { version: "1.2.5", expected: true },
                    { version: "1.2.6+build", expected: true },
                    { version: "1.3.2", expected: false },
                    { version: "1.3.3", expected: false },
                ],
            },
        ];

        groups.forEach(function (group) {
            const range = VersionRange.fromRequirementString(
                group.requirements
            );
            describe(`with requirements '${group.requirements}'`, function () {
                group.tests.forEach(function (test) {
                    it(`returns ${test.expected} for '${test.version}'`, function () {
                        const version = Version.fromSemver(test.version);
                        const result = range.satisfiedBy.apply(range, [
                            version,
                        ]);
                        assert.equal(result, test.expected);
                    });
                });
            });
        });
    });
});

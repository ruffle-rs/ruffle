import { strict as assert } from "assert";
import { mergeSorted } from "../src/internal/register-element";

describe("mergeSorted", function () {
    const numComparator = function (x: number, y: number): number {
        return x - y;
    };

    it("should merge two evenly sized sorted arrays", function () {
        assert.deepEqual(
            mergeSorted([1, 3, 5], [2, 4, 6], numComparator),
            [1, 2, 3, 4, 5, 6],
        );
    });

    it("should handle arrays of different lengths", function () {
        assert.deepEqual(
            mergeSorted([1, 2], [3, 4, 5, 6], numComparator),
            [1, 2, 3, 4, 5, 6],
        );
        assert.deepEqual(
            mergeSorted([1, 3, 5, 7], [2, 4], numComparator),
            [1, 2, 3, 4, 5, 7],
        );
    });

    it("should handle an empty first array", function () {
        assert.deepEqual(mergeSorted([], [1, 2, 3], numComparator), [1, 2, 3]);
    });

    it("should handle an empty second array", function () {
        assert.deepEqual(mergeSorted([1, 2, 3], [], numComparator), [1, 2, 3]);
    });

    it("should handle both arrays being empty", function () {
        assert.deepEqual(mergeSorted([], [], numComparator), []);
    });

    it("should handle duplicates and overlapping values", function () {
        assert.deepEqual(
            mergeSorted([1, 2, 2, 5], [2, 3, 5, 7], numComparator),
            [1, 2, 2, 2, 3, 5, 5, 7],
        );
    });

    it("should maintain stability and order when elements are equal", function () {
        interface Item {
            id: number;
            source: string;
        }
        const itemComparator = function (x: Item, y: Item): number {
            return x.id - y.id;
        };

        const a: Item[] = [
            { id: 1, source: "a" },
            { id: 2, source: "a" },
        ];
        const b: Item[] = [
            { id: 2, source: "b" },
            { id: 3, source: "b" },
        ];

        // Since the comparator returns 0 for id: 2, the <= 0 condition in
        // mergeSorted ensures that the element from array 'a' comes first.
        assert.deepEqual(mergeSorted(a, b, itemComparator), [
            { id: 1, source: "a" },
            { id: 2, source: "a" },
            { id: 2, source: "b" },
            { id: 3, source: "b" },
        ]);
    });
});

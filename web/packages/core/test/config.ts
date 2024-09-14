import { strict as assert } from "assert";
import { parseColor, parseDuration } from "../src/internal/builder";

describe("Color parsing", function () {
    it("should parse a valid RRGGBB hex, with hash", function () {
        assert.strictEqual(parseColor("#A1B2C3"), 0xa1b2c3);
    });

    it("should parse a valid RRGGBB hex, without hash", function () {
        assert.strictEqual(parseColor("1A2B3C"), 0x1a2b3c);
    });

    it("should fail with not enough digits", function () {
        assert.strictEqual(parseColor("123"), undefined);
    });

    it("should treat invalid hex as 0", function () {
        assert.strictEqual(parseColor("#AX2Y3Z"), 0xa02030);
    });
});

describe("Duration parsing", function () {
    it("should accept number of seconds as number", function () {
        assert.strictEqual(parseDuration(12.3), 12.3);
    });

    it("should accept a legacy style duration", function () {
        assert.strictEqual(parseDuration({ secs: 12.3, nanos: 400000 }), 12.3);
    });
});

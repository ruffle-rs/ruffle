import { strict as assert } from "assert";
import { swfFileName } from "../src/swf-file-name";

describe("swfFileName", function () {
    it("should extract simple SWF name", function () {
        assert.deepEqual(
            swfFileName("http://example.com/file.swf"),
            "file.swf"
        );
    });
    it("should not include query parameters", function () {
        assert.deepEqual(
            swfFileName(
                "https://uploads.ungrounded.net/574000/574241_DiamondNGSP.swf?123"
            ),
            "574241_DiamondNGSP.swf"
        );
    });
});

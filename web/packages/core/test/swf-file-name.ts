import { strict as assert } from "assert";
import { swfFileName } from "../src/swf-file-name";

describe("swfFileName", function () {
    it("should extract simple SWF name", function () {
        assert.deepEqual(nameFor("http://example.com/file.swf"), "file.swf");
    });
    it("should not include query parameters", function () {
        assert.deepEqual(
            nameFor(
                "https://uploads.ungrounded.net/574000/574241_DiamondNGSP.swf?123"
            ),
            "574241_DiamondNGSP.swf"
        );
    });
});

function nameFor(url: string): string {
    return swfFileName(new URL(url));
}

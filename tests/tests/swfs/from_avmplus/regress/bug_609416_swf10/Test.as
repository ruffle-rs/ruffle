/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


// var SECTION = "regress_609416";
// var VERSION = "AS3";
// var TITLE   = "encodeURIComponent and decodeURIComponent give wrong output when input contains surrogate pairs";
// var bug = "609416";



var src:String = String.fromCharCode(0xD842, 0xDF9F);
import com.adobe.test.Assert;
import com.adobe.test.Utils;
var encodedStr:String = encodeURIComponent(src);
var decodedStr:String = "";
var errorMsgStr:String = "no error";

try {
    decodedStr = decodeURIComponent(encodedStr);
} catch(e:Error) {
    errorMsgStr = e.toString();
}

if (false) {
    Assert.expectEq(
        "transcode UTF16 to UTF8 when the UTF16 code points contain a surrogate pair",
        src,
        decodedStr);
} else {
    Assert.expectEq(
        "transcode UTF16 to UTF8 when the UTF16 code points contain a surrogate pair",
        "URIError: Error #1052",
        Utils.parseError(errorMsgStr,"URIError: Error #1052".length));
}







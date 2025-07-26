/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.11";
//     var VERSION = "";
//     var TITLE   = "String.prototype.replace https://bugzilla.mozilla.org/show_bug.cgi?id=479786";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var s = "foo\n     bar\n\n    baz\n";
    var expect = " * foo\n * bar\n * \n * baz\n";

    // use escape() to produce a single-line output without using replace() again
    array[item++] = Assert.expectEq(
                                    "s.replace()",
                                    escape(expect),
                                    escape(s.replace(/^ */gm, " * ")) );

    return array;

}

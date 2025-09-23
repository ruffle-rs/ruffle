/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 *
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=558863
 *
 */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

//-----------------------------------------------------------------------------

// var SECTION = "regress_558863";
// var VERSION = "AS3";
// var TITLE   = "bytearray in operator throws exception for non-natural number";
// var bug = "558863";

import com.adobe.test.Assert;
var testcases = getTestCases();

function getTestCases() {

    import avmplus.System;
    import flash.utils.ByteArray;

    function checkExn( x, thunk, expectExn )
    {
        try {
            var retval = thunk();
            return !expectExn && retval;
        } catch (e) {
            // trace("threw up on "+typeof(x)+" "+x);
            return expectExn;
        }
    }

    var b = new ByteArray()
    b[0] = 1
    b[2] = 1

    function trial( x, expect, expectExn )
    {
        var f = function () { return Boolean(int(!expect) ^ int(x in b)) };
        return checkExn( x, f, expectExn );
    }

    function btest_pre_bugfix()
    {
        return (trial(0, true,  false)
                && trial(1, true,  false)
                && trial(2, true,  false)
                && trial(3, false, false)

                // non-naturals below
                && trial(true, false, true)
                && trial("xy", false, true)
                && trial({},   false, true)
                && trial(5.5,  false, true)
                && trial(-5,   false, true))
    }

    function btest_post_bugfix()
    {
        return (trial(0, true,  false)
                && trial(1, true,  false)
                && trial(2, true,  false)
                && trial(3, false, false)

                // non-naturals below
                && trial(true, false, false)
                && trial("xy", false, false)
                && trial({},   false, false)
                && trial(5.5,  false, false)
                && trial(-5,   false, false))
    }

    var array:Array = new Array();
    if (false) {
        var status = "Verify byte-array in does not throw";
        var actual = btest_post_bugfix();
        var expect = true;
        array[0] = Assert.expectEq( status, expect, actual)
    } else {
        var status = "Verify byte-array in throws when expected";
        var actual = btest_pre_bugfix();
        var expect = true;
        array[0] = Assert.expectEq( status, expect, actual)
    }
    return array;
}

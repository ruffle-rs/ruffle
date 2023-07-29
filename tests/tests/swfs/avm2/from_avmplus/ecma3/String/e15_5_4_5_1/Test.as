/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.5-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.charCodeAt";


    var TEST_STRING = new String( " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~" );

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();

    for ( j = 0, i = 0x0020; i < 0x007e; i++, j++ ) {
        array[j] = Assert.expectEq(  "TEST_STRING.charCodeAt("+j+")", i, TEST_STRING.charCodeAt( j ) );
    }

    item = array.length;

    array[item++] = Assert.expectEq(  'TEST_STRING.charCodeAt('+i+')', NaN,    TEST_STRING.charCodeAt( i ) );
    return array;
}

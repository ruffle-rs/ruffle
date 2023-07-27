/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11.11";
//     var VERSION = "ECMA_1";
    var BUGNUMBER="771111";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(     "void 0 && void 0",        undefined,   void 0 && void 0 );
    array[item++] = Assert.expectEq(     "null && null",          null,   null && null );
    array[item++] = Assert.expectEq(     "0 && 0",        0,   0 && 0 );
    array[item++] = Assert.expectEq(     "1 && 1",          1,   1 && 1 );
    array[item++] = Assert.expectEq(     "-1 && -1",          -1,   -1 && -1 );
    array[item++] = Assert.expectEq(     "54 && 54",          54,   54 && 54 );
    array[item++] = Assert.expectEq(     "54 && 45",          45,   54 && 45 );
    array[item++] = Assert.expectEq(     "true && true",          true,   true && true );
    array[item++] = Assert.expectEq(     "true && false",          false,   true && false );
    array[item++] = Assert.expectEq(     "false && true",          false,   false && true );
    array[item++] = Assert.expectEq(     "false && false",          false,   false && false );
    array[item++] = Assert.expectEq(     "0 && true",          0,   0 && true );
    array[item++] = Assert.expectEq(     "true && 0",          0,   true && 0 );
    array[item++] = Assert.expectEq(     "true && 1",          1,   true && 1 );
    array[item++] = Assert.expectEq(     "1 && true",          true,   1 && true );
    array[item++] = Assert.expectEq(     "-1 && true",          true,   -1 && true );
    array[item++] = Assert.expectEq(     "true && -1",          -1,   true && -1 );
    array[item++] = Assert.expectEq(     "true && 9",          9,   true && 9 );
    array[item++] = Assert.expectEq(     "true && -9",          -9,   true && -9 );
    array[item++] = Assert.expectEq(     "9 && true",          true,   9 && true );
    array[item++] = Assert.expectEq(     "true && Number.POSITIVE_INFINITY",          Number.POSITIVE_INFINITY,   true && Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY && true",          true,   Number.NEGATIVE_INFINITY && true );
    array[item++] = Assert.expectEq(     "true && 'str'",          "str",   true && "str" );
    array[item++] = Assert.expectEq(     "'str' && true",          true,   "str" && true);
    array[item++] = Assert.expectEq(     "false && 'str'",          false,   false && "str" );
    array[item++] = Assert.expectEq(     "'str' && false",          false,   "str" && false);
    array[item++] = Assert.expectEq(     "NaN && NaN",             NaN,  Number.NaN && Number.NaN );
    array[item++] = Assert.expectEq(     "NaN && 0",               NaN,  Number.NaN && 0 );
    array[item++] = Assert.expectEq(     "0 && NaN",               0,  0 && Number.NaN );
    array[item++] = Assert.expectEq(     "NaN && Infinity",        NaN,  Number.NaN && Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Infinity && NaN",        NaN,  Number.POSITIVE_INFINITY && Number.NaN );

    array[item++] = Assert.expectEq(     "void 0 || void 0",        undefined,   void 0 || void 0 );
    array[item++] = Assert.expectEq(     "null || null",          null,   null || null );
    array[item++] = Assert.expectEq(     "0 || 0",        0,   0 || 0 );
    array[item++] = Assert.expectEq(     "1 || 1",          1,   1 || 1 );
    array[item++] = Assert.expectEq(     "-1 || -1",          -1,   -1 || -1 );
    array[item++] = Assert.expectEq(     "54 || 54",          54,   54 || 54 );
    array[item++] = Assert.expectEq(     "54 || 45",          54,   54 || 45 );
    array[item++] = Assert.expectEq(     "true || true",          true,   true || true );
    array[item++] = Assert.expectEq(     "true || false",          true,   true || false );
    array[item++] = Assert.expectEq(     "false || true",          true,   false || true );
    array[item++] = Assert.expectEq(     "false || false",          false,   false ||false );
    array[item++] = Assert.expectEq(     "0 || true",          true,   0 || true );
    array[item++] = Assert.expectEq(     "true || 0",          true,   true || 0 );
    array[item++] = Assert.expectEq(     "true || 1",          true,   true || 1 );
    array[item++] = Assert.expectEq(     "1 || true",          1,   1 || true );
    array[item++] = Assert.expectEq(     "-1 || true",          -1,   -1 || true );
    array[item++] = Assert.expectEq(     "true || -1",          true,   true || -1 );
    array[item++] = Assert.expectEq(     "true || 9",          true,   true || 9 );
    array[item++] = Assert.expectEq(     "true || -9",          true,   true || -9 );
    array[item++] = Assert.expectEq(     "9 || true",          9,   9 || true );
    array[item++] = Assert.expectEq(     "true || Number.POSITIVE_INFINITY",          true,   true || Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY || true",          Number.NEGATIVE_INFINITY,   Number.NEGATIVE_INFINITY ||true );
    array[item++] = Assert.expectEq(     "true || 'str'",          true,   true || "str" );
    array[item++] = Assert.expectEq(     "'str'|| true",          "str",   "str" || true);
    array[item++] = Assert.expectEq(     "false || 'str'",          "str",   false ||"str" );
    array[item++] = Assert.expectEq(     "'str' || false",         "str",   "str" || false);
    array[item++] = Assert.expectEq(     "NaN || NaN",             NaN,  Number.NaN || Number.NaN );
    array[item++] = Assert.expectEq(     "NaN || 0",              0,  Number.NaN || 0 );
    array[item++] = Assert.expectEq(     "0 || NaN",               NaN,  0 || Number.NaN );
    array[item++] = Assert.expectEq(     "NaN || Infinity",       Infinity,  Number.NaN ||Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Infinity || NaN",        Infinity,  Number.POSITIVE_INFINITY || Number.NaN );

    return ( array );
}

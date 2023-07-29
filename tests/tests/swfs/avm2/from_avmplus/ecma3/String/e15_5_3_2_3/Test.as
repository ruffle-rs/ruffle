/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.3.2-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.fromCharCode()";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    for ( CHARCODE = 0; CHARCODE < 256; CHARCODE++ ) {
        array[item++] = Assert.expectEq(   
                                        "(String.fromCharCode(" + CHARCODE +")).charCodeAt(0)",
                                        ToUint16(CHARCODE),
                                        (String.fromCharCode(CHARCODE)).charCodeAt(0)
                                     );
    }
    for ( CHARCODE = 256; CHARCODE < 65536; CHARCODE+=333 ) {
        array[item++] = Assert.expectEq(   
                                        "(String.fromCharCode(" + CHARCODE +")).charCodeAt(0)",
                                        ToUint16(CHARCODE),
                                        (String.fromCharCode(CHARCODE)).charCodeAt(0)
                                     );
    }
    for ( CHARCODE = 65535; CHARCODE < 65538; CHARCODE++ ) {
        array[item++] = Assert.expectEq(   
                                        "(String.fromCharCode(" + CHARCODE +")).charCodeAt(0)",
                                        ToUint16(CHARCODE),
                                        (String.fromCharCode(CHARCODE)).charCodeAt(0)
                                     );
    }
    for ( CHARCODE = Math.pow(2,32)-1; CHARCODE < Math.pow(2,32)+1; CHARCODE++ ) {
        array[item++] = Assert.expectEq(   
                                        "(String.fromCharCode(" + CHARCODE +")).charCodeAt(0)",
                                        ToUint16(CHARCODE),
                                        (String.fromCharCode(CHARCODE)).charCodeAt(0)
                                     );
    }
    for ( CHARCODE = 0; CHARCODE > -65536; CHARCODE-=3333 ) {
        array[item++] = Assert.expectEq(   
                                        "(String.fromCharCode(" + CHARCODE +")).charCodeAt(0)",
                                        ToUint16(CHARCODE),
                                        (String.fromCharCode(CHARCODE)).charCodeAt(0)
                                     );
    }
    array[item++] = Assert.expectEq(  "(String.fromCharCode(65535)).charCodeAt(0)",    65535,  (String.fromCharCode(65535)).charCodeAt(0) );
    array[item++] = Assert.expectEq(  "(String.fromCharCode(65536)).charCodeAt(0)",    0,      (String.fromCharCode(65536)).charCodeAt(0) );
    array[item++] = Assert.expectEq(  "(String.fromCharCode(65537)).charCodeAt(0)",    1,      (String.fromCharCode(65537)).charCodeAt(0) );

     return array;
}
function ToUint16( num ) {
    num = Number( num );
    if ( isNaN( num ) || num == 0 || num == Number.POSITIVE_INFINITY || num == Number.NEGATIVE_INFINITY ) {
        return 0;
    }

    var sign = ( num < 0 ) ? -1 : 1;

    num = sign * Math.floor( Math.abs( num ) );
    num = num % Math.pow(2,16);
    num = ( num > -65536 && num < 0) ? 65536 + num : num;
    return num;
}


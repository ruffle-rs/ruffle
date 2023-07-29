/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var VERSION = "0697";
//     var SECTION = "15.5.4.5-4";


    var testcases = getTestCases();

//  all tests must call a function that returns an array of TestCase objects.

function getTestCases() {
    var array = new Array();
    var MAXCHARCODE = Math.pow(2,16);
    var item=0, CHARCODE;

    for ( CHARCODE=0; CHARCODE <256; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                                     "(String.fromCharCode("+CHARCODE+")).charCodeAt(0)",
                                     CHARCODE,
                                     (String.fromCharCode(CHARCODE)).charCodeAt(0) );
    }
    for ( CHARCODE=256; CHARCODE < 65536; CHARCODE+=999 ) {
        array[item++] = Assert.expectEq( 
                                     "(String.fromCharCode("+CHARCODE+")).charCodeAt(0)",
                                     CHARCODE,
                                     (String.fromCharCode(CHARCODE)).charCodeAt(0) );
    }

    array[item++] = Assert.expectEq(  "(String.fromCharCode(65535)).charCodeAt(0)", 65535,     (String.fromCharCode(65535)).charCodeAt(0) );

    return ( array );
}

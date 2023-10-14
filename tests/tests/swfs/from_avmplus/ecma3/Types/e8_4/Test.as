/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "8.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The String type";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq( 
                                    "var s = ''; s.length",
                                    0,
                                    (s = '', s.length ) );

    array[item++] = Assert.expectEq( 
                                    "var s = ''; s.charAt(0)",
                                    "",
                                    (s = '', s.charAt(0) ) );


    for ( var i = 0x0041, TEST_STRING = "", EXPECT_STRING = ""; i < 0x007B; i++ ) {
        TEST_STRING += ("\\u"+ DecimalToHexString( i ) );
        EXPECT_STRING += String.fromCharCode(i);
    }
    //TEST_STRING = '\u0041\u0042\u0043\u0044\u0045\u0046\u0047\u0048\u0049\u004A\u004B\u004C\u004D\u004E\u004F\u0050\u0051\u0052\u0053\u0054\u0055\u0056\u0057\u0058\u0059\u005A\u005B\u005C\u005D\u005E\u005F\u0060\u0061\u0062\u0063\u0064\u0065\u0066\u0067\u0068\u0069\u006A\u006B\u006C\u006D\u006E\u006F\u0070\u0071\u0072\u0073\u0074\u0075\u0076\u0077\u0078\u0079\u007A'

    array[item++] = Assert.expectEq( 
                                    "var s = '" + TEST_STRING+ "'; s",
                                    EXPECT_STRING,
                                    (s = '\u0041\u0042\u0043\u0044\u0045\u0046\u0047\u0048\u0049\u004A\u004B\u004C\u004D\u004E\u004F\u0050\u0051\u0052\u0053\u0054\u0055\u0056\u0057\u0058\u0059\u005A\u005B\u005C\u005D\u005E\u005F\u0060\u0061\u0062\u0063\u0064\u0065\u0066\u0067\u0068\u0069\u006A\u006B\u006C\u006D\u006E\u006F\u0070\u0071\u0072\u0073\u0074\u0075\u0076\u0077\u0078\u0079\u007A', s ) );

    array[item++] = Assert.expectEq( 
                                    "var s = '" + TEST_STRING+ "'; s.length",
                                    0x007B-0x0041,
                                    (s = '\u0041\u0042\u0043\u0044\u0045\u0046\u0047\u0048\u0049\u004A\u004B\u004C\u004D\u004E\u004F\u0050\u0051\u0052\u0053\u0054\u0055\u0056\u0057\u0058\u0059\u005A\u005B\u005C\u005D\u005E\u005F\u0060\u0061\u0062\u0063\u0064\u0065\u0066\u0067\u0068\u0069\u006A\u006B\u006C\u006D\u006E\u006F\u0070\u0071\u0072\u0073\u0074\u0075\u0076\u0077\u0078\u0079\u007A', s.length ) );

    return array;
}
function DecimalToHexString( n ) {
    n = Number( n );
    var h = "";

    for ( var i = 3; i >= 0; i-- ) {
        if ( n >= Math.pow(16, i) ){
            var t = Math.floor( n  / Math.pow(16, i));
            n -= t * Math.pow(16, i);
            if ( t >= 10 ) {
                if ( t == 10 ) {
                    h += "A";
                }
                if ( t == 11 ) {
                    h += "B";
                }
                if ( t == 12 ) {
                    h += "C";
                }
                if ( t == 13 ) {
                    h += "D";
                }
                if ( t == 14 ) {
                    h += "E";
                }
                if ( t == 15 ) {
                    h += "F";
                }
            } else {
                h += String( t );
            }
        } else {
            h += "0";
        }
    }

    return h;
}

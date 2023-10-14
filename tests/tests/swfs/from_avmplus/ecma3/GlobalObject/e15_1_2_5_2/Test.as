/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.1.2.5-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "unescape(string)";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // since there is only one character following "%", no conversion should occur.

    for ( var CHARCODE = 0; CHARCODE < 256; CHARCODE += 16 ) {
        array[item++] = Assert.expectEq( 
                            "unescape( %"+ (ToHexString(CHARCODE)).substring(0,1) +" )",
                            "%"+(ToHexString(CHARCODE)).substring(0,1),
                            unescape( "%" + (ToHexString(CHARCODE)).substring(0,1) )  );
    }

    // since there is only one character following "%u", no conversion should occur.

    for ( var CHARCODE = 0; CHARCODE < 256; CHARCODE +=16 ) {
        array[item++] = Assert.expectEq( 
                            "unescape( %u"+ (ToHexString(CHARCODE)).substring(0,1) +" )",
                            "%u"+(ToHexString(CHARCODE)).substring(0,1),
                            unescape( "%u" + (ToHexString(CHARCODE)).substring(0,1) )  );
    }


    // three char unicode string.  no conversion should occur

    for ( var CHARCODE = 1024; CHARCODE < 65536; CHARCODE+= 1234 ) {
        array[item++] = Assert.expectEq
                        (    
                            "unescape( %u"+ (ToUnicodeString(CHARCODE)).substring(0,3)+ " )",

                            "%u"+(ToUnicodeString(CHARCODE)).substring(0,3),
                            unescape( "%u"+(ToUnicodeString(CHARCODE)).substring(0,3) )
                        );
    }

    return ( array );
}

function ToUnicodeString( n ) {
    var string = ToHexString(n);

    for ( var PAD = (4 - string.length ); PAD > 0; PAD-- ) {
        string = "0" + string;
    }

    return string;
}
function ToHexString( n ) {
    var hex = new Array();

    for ( var mag = 1; Math.pow(16,mag) <= n ; mag++ ) {
        ;
    }

    for ( index = 0, mag -= 1; mag > 0; index++, mag-- ) {
        hex[index] = Math.floor( n / Math.pow(16,mag) );
        n -= Math.pow(16,mag) * Math.floor( n/Math.pow(16,mag) );
    }

    hex[hex.length] = n % 16;

    var string ="";

    for ( var index = 0 ; index < hex.length ; index++ ) {
        switch ( hex[index] ) {
            case 10:
                string += "A";
                break;
            case 11:
                string += "B";
                break;
            case 12:
                string += "C";
                break;
            case 13:
                string += "D";
                break;
            case 14:
                string += "E";
                break;
            case 15:
                string += "F";
                break;
            default:
                string += hex[index];
        }
    }

    if ( string.length == 1 ) {
        string = "0" + string;
    }
    return string;
}

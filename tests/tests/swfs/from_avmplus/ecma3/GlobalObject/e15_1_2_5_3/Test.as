/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15.1.2.5-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "unescape(string)";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    for ( var CHARCODE = 0, NONHEXCHARCODE = 0; CHARCODE < 256; CHARCODE++, NONHEXCHARCODE++ ) {
        NONHEXCHARCODE = getNextNonHexCharCode( NONHEXCHARCODE );

        array[item++] = Assert.expectEq( 
                            "unescape( %"+ (ToHexString(CHARCODE)).substring(0,1) +
                                String.fromCharCode( NONHEXCHARCODE ) +" )" +
                                "[where last character is String.fromCharCode("+NONHEXCHARCODE+")]",
                            "%"+(ToHexString(CHARCODE)).substring(0,1)+
                                String.fromCharCode( NONHEXCHARCODE ),
                            unescape( "%" + (ToHexString(CHARCODE)).substring(0,1)+
                                String.fromCharCode( NONHEXCHARCODE ) )  );
    }
    for ( var CHARCODE = 0, NONHEXCHARCODE = 0; CHARCODE < 256; CHARCODE++, NONHEXCHARCODE++ ) {
        NONHEXCHARCODE = getNextNonHexCharCode( NONHEXCHARCODE );

        array[item++] = Assert.expectEq( 
                            "unescape( %u"+ (ToHexString(CHARCODE)).substring(0,1) +
                                String.fromCharCode( NONHEXCHARCODE ) +" )" +
                                "[where last character is String.fromCharCode("+NONHEXCHARCODE+")]",
                            "%u"+(ToHexString(CHARCODE)).substring(0,1)+
                                String.fromCharCode( NONHEXCHARCODE ),
                            unescape( "%u" + (ToHexString(CHARCODE)).substring(0,1)+
                                String.fromCharCode( NONHEXCHARCODE ) )  );
    }

    for ( var CHARCODE = 0, NONHEXCHARCODE = 0 ; CHARCODE < 65536; CHARCODE+= 54321, NONHEXCHARCODE++ ) {
        NONHEXCHARCODE = getNextNonHexCharCode( NONHEXCHARCODE );
        var x = "0x" + (ToUnicodeString(CHARCODE)).substring(0,2);
        array[item++] = Assert.expectEq( 
                            "unescape( %u"+ (ToUnicodeString(CHARCODE)).substring(0,3) +
                                String.fromCharCode( NONHEXCHARCODE ) +" )" +
                                "[where last character is String.fromCharCode("+NONHEXCHARCODE+")]",

                            String.fromCharCode(x) +
                            (ToUnicodeString(CHARCODE)).substring(2,3) +
                                String.fromCharCode( NONHEXCHARCODE ),

                            unescape( "%" + (ToUnicodeString(CHARCODE)).substring(0,3)+
                                String.fromCharCode( NONHEXCHARCODE ) )  );
    }

    return ( array );
}
function getNextNonHexCharCode( n ) {
    for (  ; n < Math.pow(2,16); n++ ) {
        if ( (  n == 43 || n == 45 || n == 46 || n == 47 ||
            (n >= 71 && n <= 90) || (n >= 103 && n <= 122) ||
            n == 64 || n == 95 ) ) {
            break;
        } else {
            n = ( n > 122 ) ? 0 : n;
        }
    }
    return n;
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

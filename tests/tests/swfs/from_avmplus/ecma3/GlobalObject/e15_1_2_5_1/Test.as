/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.1.2.5-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "unescape(string)";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "unescape.length",       1,               unescape.length );
    //array[item++] = Assert.expectEq(  "unescape.length = null; unescape.length",   1,      eval("unescape.length=null; unescape.length") );

    var thisError:String
    try
    {
        unescape.length=null;
    }
    catch(e:ReferenceError)
    {
        thisError = e.toString();
    }
    finally
    {
        array[item++] = Assert.expectEq( "unescape.length = null", "ReferenceError: Error #1074", Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(  "delete unescape.length",                    false,  delete unescape.length );
    //array[item++] = Assert.expectEq(  "delete unescape.length; unescape.length",   1,      eval("delete unescape.length; unescape.length") );
    delete unescape.length;
    array[item++] = Assert.expectEq(  "delete unescape.length; unescape.length",   1,      unescape.length);

    var MYPROPS='';
    for ( var p in unescape ) {
        MYPROPS+= p;
    }

    array[item++] = Assert.expectEq(  "var MYPROPS='', for ( var p in unescape ) { MYPROPS+= p }, MYPROPS",    "", MYPROPS );

    array[item++] = Assert.expectEq(  "unescape()",              "undefined",    unescape() );
    array[item++] = Assert.expectEq(  "unescape('')",            "",             unescape('') );
    array[item++] = Assert.expectEq(  "unescape( null )",        "null",         unescape(null) );
    array[item++] = Assert.expectEq(  "unescape( void 0 )",      "null",    unescape(void 0) );
    array[item++] = Assert.expectEq(  "unescape( true )",        "true",         unescape( true ) );
    array[item++] = Assert.expectEq(  "unescape( false )",       "false",        unescape( false ) );

    array[item++] = Assert.expectEq(  "unescape( new Boolean(true) )",   "true", unescape(new Boolean(true)) );
    array[item++] = Assert.expectEq(  "unescape( new Boolean(false) )",  "false",    unescape(new Boolean(false)) );

    array[item++] = Assert.expectEq(  "unescape( Number.NaN  )",                 "NaN",      unescape(Number.NaN) );
    array[item++] = Assert.expectEq(  "unescape( -0 )",                          "0",        unescape( -0 ) );
    array[item++] = Assert.expectEq(  "unescape( 'Infinity' )",                  "Infinity", unescape( "Infinity" ) );
    array[item++] = Assert.expectEq(  "unescape( Number.POSITIVE_INFINITY )",    "Infinity", unescape( Number.POSITIVE_INFINITY ) );
    array[item++] = Assert.expectEq(  "unescape( Number.NEGATIVE_INFINITY )",    "-Infinity", unescape( Number.NEGATIVE_INFINITY ) );

    var ASCII_TEST_STRING = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789@*_+-./";

    array[item++] = Assert.expectEq(  "unescape( " +ASCII_TEST_STRING+" )",    ASCII_TEST_STRING,  unescape( ASCII_TEST_STRING ) );

    // escaped chars with ascii values less than 256

    for ( var CHARCODE = 0; CHARCODE < 256; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "unescape( %"+ ToHexString(CHARCODE)+" )",
                            String.fromCharCode(CHARCODE),
                            unescape( "%" + ToHexString(CHARCODE) )  );
    }

    // unicode chars represented by two hex digits
    for ( var CHARCODE = 0; CHARCODE < 256; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "unescape( %u"+ ToHexString(CHARCODE)+" )",
                            "%u"+ToHexString(CHARCODE),
                            unescape( "%u" + ToHexString(CHARCODE) )  );
    }
/*
    for ( var CHARCODE = 0; CHARCODE < 256; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "unescape( %u"+ ToUnicodeString(CHARCODE)+" )",
                            String.fromCharCode(CHARCODE),
                            unescape( "%u" + ToUnicodeString(CHARCODE) )  );
    }
    for ( var CHARCODE = 256; CHARCODE < 65536; CHARCODE+= 333 ) {
        array[item++] = Assert.expectEq( 
                            "unescape( %u"+ ToUnicodeString(CHARCODE)+" )",
                            String.fromCharCode(CHARCODE),
                            unescape( "%u" + ToUnicodeString(CHARCODE) )  );
    }
*/
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

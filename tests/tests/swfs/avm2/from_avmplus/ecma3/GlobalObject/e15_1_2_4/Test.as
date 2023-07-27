/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.1.2.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "escape(string)";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "escape.length",         1,          escape.length );

    var thisError:String = "no error";
    try
    {
        escape.length = null;
    }
    catch (e:ReferenceError)
    {
        thisError = e.toString();
    }
    finally
    {
        array[item++] = Assert.expectEq( "escape.length = null", "ReferenceError: Error #1074", Utils.referenceError(thisError));
    }

    array[item++] = Assert.expectEq(  "delete escape.length",                  false,  delete escape.length );
    delete escape.length;
    array[item++] = Assert.expectEq(  "delete escape.length; escape.length",   1, escape.length);

    var MYPROPS='';
    for ( var p in escape ) {
        MYPROPS+= p;
    }

    array[item++] = Assert.expectEq(  "var MYPROPS='', for ( var p in escape ) { MYPROPS+= p}, MYPROPS",    "",    MYPROPS );

    array[item++] = Assert.expectEq(  "escape()",              "undefined",    escape() );
    array[item++] = Assert.expectEq(  "escape('')",            "",             escape('') );
    array[item++] = Assert.expectEq(  "escape( null )",        "null",         escape(null) );
    array[item++] = Assert.expectEq(  "escape( void 0 )",      "null",    escape(void 0) );
    array[item++] = Assert.expectEq(  "escape( true )",        "true",         escape( true ) );
    array[item++] = Assert.expectEq(  "escape( false )",       "false",        escape( false ) );

    array[item++] = Assert.expectEq(  "escape( new Boolean(true) )",   "true", escape(new Boolean(true)) );
    array[item++] = Assert.expectEq(  "escape( new Boolean(false) )",  "false",    escape(new Boolean(false)) );

    array[item++] = Assert.expectEq(  "escape( Number.NaN  )",                 "NaN",      escape(Number.NaN) );
    array[item++] = Assert.expectEq(  "escape( -0 )",                          "0",        escape( -0 ) );
    array[item++] = Assert.expectEq(  "escape( 'Infinity' )",                  "Infinity", escape( "Infinity" ) );
    array[item++] = Assert.expectEq(  "escape( Number.POSITIVE_INFINITY )",    "Infinity", escape( Number.POSITIVE_INFINITY ) );
    array[item++] = Assert.expectEq(  "escape( Number.NEGATIVE_INFINITY )",    "-Infinity", escape( Number.NEGATIVE_INFINITY ) );

    var ASCII_TEST_STRING = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789@*_+-./";

    array[item++] = Assert.expectEq(  "escape( " +ASCII_TEST_STRING+" )",    ASCII_TEST_STRING,  escape( ASCII_TEST_STRING ) );

    // ASCII value less than

    for ( var CHARCODE = 0; CHARCODE < 32; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "escape(String.fromCharCode("+CHARCODE+"))",
                            "%"+ToHexString(CHARCODE),
                            escape(String.fromCharCode(CHARCODE))  );
    }
    for ( var CHARCODE = 128; CHARCODE < 256; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "escape(String.fromCharCode("+CHARCODE+"))",
                            "%"+ToHexString(CHARCODE),
                            escape(String.fromCharCode(CHARCODE))  );
    }

    for ( var CHARCODE = 256; CHARCODE < 1024; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "escape(String.fromCharCode("+CHARCODE+"))",
                            "%u"+ ToUnicodeString(CHARCODE),
                            escape(String.fromCharCode(CHARCODE))  );
    }
    for ( var CHARCODE = 65500; CHARCODE < 65536; CHARCODE++ ) {
        array[item++] = Assert.expectEq( 
                            "escape(String.fromCharCode("+CHARCODE+"))",
                            "%u"+ ToUnicodeString(CHARCODE),
                            escape(String.fromCharCode(CHARCODE))  );
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

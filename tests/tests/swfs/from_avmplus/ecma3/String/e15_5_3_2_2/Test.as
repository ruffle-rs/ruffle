/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.3.2-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.fromCharCode()";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    

    var args='';
    for ( i = 0x0020; i < 0x007f; i++ ) {
        args += ( i == 0x007e ) ? i : i + ', ';
    }

    //print (args);

    var MYSTRING = String.fromCharCode( 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52,
                    53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
                    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94,
                    95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
                    113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126 );
    
    array[item++] = Assert.expectEq( 
                                  "var MYSTRING = String.fromCharCode( args)",
                                  " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
                                  MYSTRING);
    
    /*
    var MYSTRING = String.fromCharCode(args);
    array[item++] = Assert.expectEq( 
                                  "MYSTRING",
                                  " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
                                  MYSTRING);
    */
    array[item++] = Assert.expectEq( 
                                    "MYSTRING.length",
                                    0x007f - 0x0020,
                                    MYSTRING.length );
     return array;
}

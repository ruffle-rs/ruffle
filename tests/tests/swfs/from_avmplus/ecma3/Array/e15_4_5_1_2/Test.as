/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

   // TODO: REVIEW AS4 CONVERSION ISSUE 
    var SECTION = "15.4.5.1-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array [[Put]] (P,V)";


    var testcases = new Array();

    var a = new Array();

    var tc = 0;

    AddCase( "3.00", "three" );
    AddCase( "00010", "eight" );
    AddCase( "37xyz", "thirty-five" );
    AddCase("5000000000", 5)
    AddCase( "-2", -3 );

    testcases[tc++] = Assert.expectEq( 
        "a[10]",
        void 0,
        a[10] );

    testcases[tc++] = Assert.expectEq( 
        "a[3]",
        void 0,
        a[3] );

    a[4] = "four";

    testcases[tc++] = Assert.expectEq( 
        "a[4] = \"four\"; a[4]",
        "four",
        a[4] );

    testcases[tc++] = Assert.expectEq( 
        "a[\"4\"]",
        "four",
        a["4"] );

    testcases[tc++] = Assert.expectEq( 
        "a[\"4.00\"]",
        void 0,
        a["4.00"] );

    testcases[tc++] = Assert.expectEq( 
        "a.length",
        5,
        a.length );


    a["5000000000"] = 5;

    testcases[tc++] = Assert.expectEq( 
        "a[\"5000000000\"] = 5; a.length",
        5,
        a.length );

    testcases[tc++] = Assert.expectEq( 
        "a[\"-2\"] = -3; a.length",
        5,
        a.length );


function AddCase ( arg, value ) {

    a[arg] = value;

    testcases[tc++] = Assert.expectEq( 
        "a[\"" + arg + "\"] =  "+ value +"; a.length",
        0,
        a.length );
}

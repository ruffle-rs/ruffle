/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "12.6.3-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The for..in statment";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;


    var o = {};

    var result = "";

    for ( o.a in [1,2,3] ) { result += String( [1,2,3][o.a] ); }

    // need to show that all all got called, not the oreder
    var myArray = new Array( 1, 2, 3 );
    var result2 = "PASSED"
    for( var x = 0; x < myArray.length; x++ ){
        if( result.indexOf( myArray[x] ) == -1 ){
            result2="FAILED";
            break;
        }
    }
    array[item++] = Assert.expectEq( 
        "for ( o.a in [1,2,3] ) { result += String( [1,2,3][o.a] ); } result",
        "PASSED",
        result2 );

    return array;
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// TODO: REVIEW AS4 CONVERSION ISSUE 
var TIME_1970    = 0;
var TIME_2000    = 946684800000;
var TIME_1900    = -2208988800000;

   // TODO: REVIEW AS4 CONVERSION ISSUE      
   var SECTION = "15.4.4.5-3";

//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.prototype.sort(comparefn)";


    var testcases = new Array();
    getTestCases();
   // test("no actual"); // don't want the actual results printed to output

function getTestCases() {
    var array = new Array();

    array[array.length] = new Date( TIME_2000 * Math.PI );
    array[array.length] = new Date( TIME_2000 * 10 );
    array[array.length] = new Date( TIME_1900 + TIME_1900  );
    array[array.length] = new Date(0);
    array[array.length] = new Date( TIME_2000 );
    array[array.length] = new Date( TIME_1900 + TIME_1900 +TIME_1900 );
    array[array.length] = new Date( TIME_1900 * Math.PI );
    array[array.length] = new Date( TIME_1900 * 10 );
    array[array.length] = new Date( TIME_1900 );
    array[array.length] = new Date( TIME_2000 + TIME_2000 );
    array[array.length] = new Date( 1899, 0, 1 );
    array[array.length] = new Date( 2000, 1, 29 );
    array[array.length] = new Date( 2000, 0, 1 );
    array[array.length] = new Date( 1999, 11, 31 );

   var testarr1 = new Array()
   clone( array, testarr1 );
   testarr1.sort( comparefn1 );

   var testarr2 = new Array()
   clone( array, testarr2 );
   testarr2.sort( comparefn2 );

    var testarr3 = new Array();
    clone( array, testarr3 );
    testarr3.sort( comparefn3 );

    // when there's no sort function, sort sorts by the toString value of Date.

   var testarr4 = new Array()
   clone( array, testarr4 );
   testarr4.sort();

    var realarr = new Array();
    clone( array, realarr );
    realarr.sort( realsort );

    var stringarr = new Array();
    clone( array, stringarr );
    stringarr.sort( stringsort );

    for ( var i = 0, tc = 0; i < array.length; i++, tc++) {
        testcases[tc] = Assert.expectEq(
    
            "testarr1["+i+"]",
            realarr[i],
            testarr1[i] );
    }

    for ( var i=0; i < array.length; i++,  tc++) {
        testcases[tc] = Assert.expectEq(

            "testarr2["+i+"]",
            realarr[i],
            testarr2[i] );
    }

    for ( var i=0; i < array.length; i++,  tc++) {
        testcases[tc] = Assert.expectEq(
   
            "testarr3["+i+"]",
            realarr[i],
            testarr3[i] );
    }

    for ( var i=0; i < array.length; i++,  tc++) {
        testcases[tc] = Assert.expectEq(
    
            "testarr4["+i+"]",
            stringarr[i].toString(),
            testarr4[i].toString() );
    }

}
function comparefn1( x, y ) {
    return x - y;
}
function comparefn2( x, y ) {
    return x.valueOf() - y.valueOf();
}
function realsort( x, y ) {
    return ( x.valueOf() == y.valueOf() ? 0 : ( x.valueOf() > y.valueOf() ? 1 : -1 ) );
}
function comparefn3( x, y ) {
    return ( x == y ? 0 : ( x > y ? 1: -1 ) );
}
function clone( source, target ) {
    for (var i = 0; i < source.length; i++ ) {
        target[i] = source[i];
    }
}
function stringsort( x, y ) {
    for ( var i = 0; i < x.toString().length; i++ ) {
        var d = (x.toString()).charCodeAt(i) - (y.toString()).charCodeAt(i);
        if ( d > 0 ) {
            return 1;
        } else {
            if ( d < 0 ) {
                return -1;
            } else {
                continue;
            }
        }

        var d = x.length - y.length;

        if  ( d > 0 ) {
            return 1;
        } else {
            if ( d < 0 ) {
                return -1;
            }
        }
   }
    return 0;
}

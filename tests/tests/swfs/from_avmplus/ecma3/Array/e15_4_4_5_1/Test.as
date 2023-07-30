/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


     var SECTION = "15.4.4.5-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.prototype.sort(comparefn)";


    var testcases = new Array();
    getTestCases();

function getTestCases() {
    var S = new Array();
    var item = 0;
    var A;

    // array is empty.
    S[item++] = (A = new Array());

    // array contains one item
    S[item++] = (A = new Array( true ));

    // length of array is 2
    S[item++] = (A = new Array( true, false, new Boolean(true), new Boolean(false), 'true', 'false' ) );

    S[item++] = (A = new Array(), A[3] = 'undefined', A[6] = null, A[8] = 'null', A[0] = void 0, A);

    /*

    S[item] = "var A = new Array( ";

    var limit = 0x0061;
    for ( var i = 0x007A; i >= limit; i-- ) {
        S[item] += "\'"+ String.fromCharCode(i) +"\'" ;
        if ( i > limit ) {
            S[item] += ",";
        }
    }
    */

    S[item] = (A = new Array(( 0x007A - 0x0061) + 1) );

    var limit = 0x0061;
    var index = 0;
    for ( var i = 0x007A; i >= limit; i-- ) {
        A[index++] = String.fromCharCode(i);
    }

    item++;

    for ( var i = 0; i < S.length; i++ ) {
        CheckItems( S[i] );
    }
}
function CheckItems( S ) {
    var A = S;
    var E = Sort( A );

    testcases[testcases.length] = Assert.expectEq(   
                                    S +";  A.sort(); A.length",
                                    E.length,
                                    (A = S, A.sort(), A.length) );

    for ( var i = 0; i < E.length; i++ ) {
        testcases[testcases.length] = Assert.expectEq( 
                                            "A["+i+ "].toString()",
                                            E[i] +"",
                                            A[i] +"");

        if ( A[i] == void 0 && typeof A[i] == "undefined" ) {
            testcases[testcases.length] = Assert.expectEq( 
                                            "typeof A["+i+ "]",
                                            typeof E[i],
                                            typeof A[i] );
        }
    }
}
function Sort( a ) {
    for ( var i = 0; i < a.length; i++ ) {
        for ( var j = i+1; j < a.length; j++ ) {
            var lo = a[i];
            var hi = a[j];
            var c = Compare( lo, hi );
            if ( c == 1 ) {
                a[i] = hi;
                a[j] = lo;
            }
        }
    }
    return a;
}
function Compare( x, y ) {
    if ( x == void 0 && y == void 0  && typeof x == "undefined" && typeof y == "undefined" ) {
        return +0;
    }
    if ( x == void 0  && typeof x == "undefined" ) {
        return 1;
    }
    if ( y == void 0 && typeof y == "undefined" ) {
        return -1;
    }
    x = String(x);
    y = String(y);
    if ( x < y ) {
        return -1;
    }
    if ( x > y ) {
        return 1;
    }
    return 0;
}

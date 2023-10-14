/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.4.4-1";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;
    var ARR_PROTOTYPE = Array.prototype;

    array[item++] = Assert.expectEq(  "Array.prototype.reverse.length",           0,      Array.prototype.reverse.length );
    array[item++] = Assert.expectEq(  "delete Array.prototype.reverse.length",    false,  delete Array.prototype.reverse.length );
    array[item++] = Assert.expectEq(  "delete Array.prototype.reverse.length; Array.prototype.reverse.length",    0, (delete Array.prototype.reverse.length, Array.prototype.reverse.length) );

    // length of array is 0
    var A;
    array[item++] = Assert.expectEq(   
                                    "var A = new Array();   A.reverse(); A.length",
                                    0,
                                    (A = new Array(),   A.reverse(), A.length ) );

    function CheckItems( R, A ) {
        for ( var i = 0; i < R.length; i++ ) {
            array[item++] = Assert.expectEq(
                                                SECTION,
                                                "A["+i+ "]",
                                                R[i],
                                                A[i] );
        }
    }

    return ( array );
}

/*function Object_1( value ) {
    this.array = value.split(",");
    this.length = this.array.length;
    for ( var i = 0; i < this.length; i++ ) {
        this[i] = this.array[i];
    }
    this.join = Array.prototype.reverse;
    this.getClass = Object.prototype.toString;
}*/
function Reverse( array ) {
    var r2 = array.length;
    var k = 0;
    var r3 = Math.floor( r2/2 );
    if ( r3 == k ) {
        return array;
    }

    for ( k = 0;  k < r3; k++ ) {
        var r6 = r2 - k - 1;
//        var r7 = String( k );
        var r7 = k;
        var r8 = String( r6 );

        var r9 = array[r7];
        var r10 = array[r8];

        array[r7] = r10;
        array[r8] = r9;
    }

    return array;
}
function Iterate( array ) {
    for ( var i = 0; i < array.length; i++ ) {
//        print( i+": "+ array[String(i)] );
    }
}

function Object_1( value ) {
    this.array = value.split(",");
    this.length = this.array.length;
    for ( var i = 0; i < this.length; i++ ) {
        this[i] = this.array[i];
    }
    this.reverse = Array.prototype.reverse;
    this.getClass = Object.prototype.toString;
}

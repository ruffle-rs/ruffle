/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.5.4.7-3";
//     var VERSION = "ECMA_2";
//     var TITLE   = "String.protoype.lastIndexOf";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var thisError="no error";
    var b = true;
    try{
        b.__proto__.lastIndexOf = String.prototype.lastIndexOf;
        b.lastIndexOf('r', 0 );
    }catch(e:Error){
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 0 )","ReferenceError: Error #1069"
,Utils.referenceError(thisError));
    }

   /* array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 0 )",
                                    -1,
                                    (b = true, b.__proto__.lastIndexOf = String.prototype.lastIndexOf, b.lastIndexOf('r', 0 ) ) );*/

    thisError="no error";
    var b = true;
    try{
        b.__proto__.lastIndexOf = String.prototype.lastIndexOf;
        b.lastIndexOf('r', 1 );
    }catch(e1:Error){
        thisError=e1.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 1 )","ReferenceError: Error #1069"
,Utils.referenceError(thisError));
    }

   /* array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 1 )",
                                    1,
                                    (b = true, b.__proto__.lastIndexOf = String.prototype.lastIndexOf, b.lastIndexOf('r', 1 ) ) );*/

    thisError="no error";
    var b = true;
    try{
        b.__proto__.lastIndexOf = String.prototype.lastIndexOf;
        b.lastIndexOf('r', 2 );
    }catch(e2:Error){
        thisError=e2.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 2 )","ReferenceError: Error #1069"
,Utils.referenceError(thisError));
    }

   /* array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 2 )",
                                    1,
                                    (b = true, b.__proto__.lastIndexOf = String.prototype.lastIndexOf, b.lastIndexOf('r', 2 ) ) );*/

    thisError="no error";
    var b = true;
    try{
        b.__proto__.lastIndexOf = String.prototype.lastIndexOf;
        b.lastIndexOf('r', 10 );
    }catch(e10:Error){
        thisError=e10.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 10 )","ReferenceError: Error #1069"
,Utils.referenceError(thisError));
    }

  /*array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r', 10 )",
                                    1,
                                    (b = true, b.__proto__.lastIndexOf = String.prototype.lastIndexOf, b.lastIndexOf('r', 10 ) ) );*/
    thisError="no error";
    var b = true;
    try{
        b.__proto__.lastIndexOf = String.prototype.lastIndexOf;
        b.lastIndexOf('r' );
    }catch(e3:Error){
        thisError=e3.toString();
    }finally{
        array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r')","ReferenceError: Error #1069"
,Utils.referenceError(thisError));
    }

    /*array[item++] = Assert.expectEq(   
                                    "var b = true; b.__proto__.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('r' )",
                                    1,
                                    (b = true, b.__proto__.lastIndexOf = String.prototype.lastIndexOf, b.lastIndexOf('r' ) ) );*/

    return array;
}

function LastIndexOf( string, search, position ) {
    string = String( string );
    search = String( search );

    position = Number( position )

    if ( isNaN( position ) ) {
        position = Infinity;
    } else {
        position = ToInteger( position );
    }

    result5= string.length;
    result6 = Math.min(Math.max(position, 0), result5);
    result7 = search.length;

    if (result7 == 0) {
        return Math.min(position, result5);
    }

    result8 = -1;

    for ( k = 0; k <= result6; k++ ) {
        if ( k+ result7 > result5 ) {
            break;
        }
        for ( j = 0; j < result7; j++ ) {
            if ( string.charAt(k+j) != search.charAt(j) ){
                break;
            }   else  {
                if ( j == result7 -1 ) {
                    result8 = k;
                }
            }
        }
    }

    return result8;
}
function ToInteger( n ) {
    n = Number( n );
    if ( isNaN(n) ) {
        return 0;
    }
    if ( Math.abs(n) == 0 || Math.abs(n) == Infinity ) {
        return n;
    }

    var sign = ( n < 0 ) ? -1 : 1;

    return ( sign * Math.floor(Math.abs(n)) );
}

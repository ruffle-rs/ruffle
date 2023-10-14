/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.7-2";
//     var VERSION = "ECMA_4";
//     var TITLE   = "String.protoype.lastIndexOf";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "String.prototype.lastIndexOf.length",           2,          String.prototype.lastIndexOf.length );
    array[item++] = Assert.expectEq(  "delete String.prototype.lastIndexOf.length",    false,      delete String.prototype.lastIndexOf.length );
    array[item++] = Assert.expectEq(  "delete String.prototype.lastIndexOf.length; String.prototype.lastIndexOf.length",  2,  (delete String.prototype.lastIndexOf.length, String.prototype.lastIndexOf.length ) );

    array[item++] = Assert.expectEq(  "var s = new String(''), s.lastIndexOf('', 0)",          LastIndexOf("","",0),  (s = new String(''), s.lastIndexOf('', 0) ) );
    array[item++] = Assert.expectEq(  "var s = new String(''), s.lastIndexOf('')",             LastIndexOf("",""),    (s = new String(''), s.lastIndexOf('') ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('', 0)",     LastIndexOf("hello","",0),  (s = new String('hello'), s.lastIndexOf('',0) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('')",        LastIndexOf("hello",""),    (s = new String('hello'), s.lastIndexOf('') ) );

    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll')",     LastIndexOf("hello","ll" ),   (s = new String('hello'), s.lastIndexOf('ll') ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 0)",  LastIndexOf("hello","ll",0),  (s = new String('hello'), s.lastIndexOf('ll', 0) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 1)",  LastIndexOf("hello","ll",1),  (s = new String('hello'), s.lastIndexOf('ll', 1) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 2)",  LastIndexOf("hello","ll",2),  (s = new String('hello'), s.lastIndexOf('ll', 2) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 3)",  LastIndexOf("hello","ll",3),  (s = new String('hello'), s.lastIndexOf('ll', 3) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 4)",  LastIndexOf("hello","ll",4),  (s = new String('hello'), s.lastIndexOf('ll', 4) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 5)",  LastIndexOf("hello","ll",5),  (s = new String('hello'), s.lastIndexOf('ll', 5) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 6)",  LastIndexOf("hello","ll",6),  (s = new String('hello'), s.lastIndexOf('ll', 6) ) );

    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 1.5)", LastIndexOf('hello','ll', 1.5), (s = new String('hello'), s.lastIndexOf('ll', 1.5) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', 2.5)", LastIndexOf('hello','ll', 2.5),  (s = new String('hello'), s.lastIndexOf('ll', 2.5) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', -1)",  LastIndexOf('hello','ll', -1), (s = new String('hello'), s.lastIndexOf('ll', -1) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', -1.5)",LastIndexOf('hello','ll', -1.5), (s = new String('hello'), s.lastIndexOf('ll', -1.5) ) );


    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', -Infinity)",    LastIndexOf("hello","ll",-Infinity), (s = new String('hello'), s.lastIndexOf('ll', -Infinity) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', Infinity)",    LastIndexOf("hello","ll",Infinity), (s = new String('hello'), s.lastIndexOf('ll', Infinity) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', NaN)",    LastIndexOf("hello","ll",NaN), (s = new String('hello'), s.lastIndexOf('ll', NaN) ) );
    array[item++] = Assert.expectEq(  "var s = new String('hello'), s.lastIndexOf('ll', -0)",    LastIndexOf("hello","ll",-0), (s = new String('hello'), s.lastIndexOf('ll', -0) ) );

    for ( var i = 0; i < ( "[object Object]" ).length; i++ ) {
        array[item++] = Assert.expectEq(   
                                        "var o = new Object(); o.lastIndexOf = String.prototype.lastIndexOf; o.lastIndexOf('b', "+ i + ")",
                                        ( i < 2 ? -1 : ( i < 9  ? 2 : 9 )) ,
                                        (o = new Object(), o.lastIndexOf = String.prototype.lastIndexOf, o.lastIndexOf('b', i )) );
    }

    var origBooleanLastIndexOf = Boolean.prototype.lastIndexOf;
    Boolean.prototype.lastIndexOf = String.prototype.lastIndexOf;
    for ( var i = 0; i < 5; i ++ ) {
        array[item++] = Assert.expectEq(   
                                        "var b = new Boolean(); b.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('l', "+ i + ")",
                                        ( i < 2 ? -1 : 2 ),
                                        (b = new Boolean(), b.lastIndexOf('l', i )) );
    }

    var origBooleanToString = Boolean.prototype.toString;
    Boolean.prototype.toString=Object.prototype.toString;
    for ( var i = 0; i < 5; i ++ ) {
        array[item++] = Assert.expectEq(   
                                        "var b = new Boolean(); b.toString = Object.prototype.toString; b.lastIndexOf = String.prototype.lastIndexOf; b.lastIndexOf('o', "+ i + ")",
                                        (-1),
                                        (b = new Boolean(),  b.lastIndexOf('o', i )) );
    }

    var origNumberLastIndexOf = Number.prototype.lastIndexOf;
    Number.prototype.lastIndexOf=String.prototype.lastIndexOf;
    for ( var i = 0; i < 9; i++ ) {
        array[item++] = Assert.expectEq(   
                                        "var n = new Number(Infinity); n.lastIndexOf = String.prototype.lastIndexOf; n.lastIndexOf( 'i', " + i + " )",
                                        ( i < 3 ? -1 : ( i < 5 ? 3 : 5 ) ),
                                        (n = new Number(Infinity), n.lastIndexOf( 'i', i ) ) );
    }
    
 //   if (!as3Enabled) {
 //       var a = new Array( "abc","def","ghi","jkl","mno","pqr","stu","vwx","yz" );
 //       a.lastIndexOf = String.prototype.lastIndexOf;
        
 //       for ( var i = 0; i < (a.toString()).length; i++ ) {
 //           array[item++] = Assert.expectEq( 
 //                                        "var a = new Array( 'abc','def','ghi','jkl','mno','pqr','stu','vwx','yz' ); a.lastIndexOf = String.prototype.lastIndexOf; a.lastIndexOf( ',mno,p', "+i+" )",
 //                                       ( i < 15 ? -1 : 15 ),
 //                                         (a.lastIndexOf( ',mno,p', i ) ) );
 //       }
 //   }

    var origMathLastIndexOf = Math.lastIndexOf;
    for ( var i = 0; i < 15; i ++ ) {
        array[item++] = Assert.expectEq(   
                                        "var m = Math; m.lastIndexOf = String.prototype.lastIndexOf; m.lastIndexOf('t', "+ i + ")",
                                        ( i < 9 ? -1 : 9 ),
                                        (m = Math, m.lastIndexOf = String.prototype.lastIndexOf, m.lastIndexOf('t', i)) );
    }

    
    
/*
    for ( var i = 0; i < 15; i++ ) {
        array[item++] = Assert.expectEq(   
                                        "var d = new Date(); d.lastIndexOf = String.prototype.lastIndexOf; d.lastIndexOf( '0' )",
                                        (d = new Date(), d.lastIndexOf = String.prototype.lastIndexOf, d.lastIndexOf( '0' ))
                                    )
    }

*/

    //restore
    Boolean.prototype.lastIndexOf = origBooleanLastIndexOf;
    Boolean.prototype.toString = origBooleanToString;
    Number.prototype.lastIndexOf = origNumberLastIndexOf;

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

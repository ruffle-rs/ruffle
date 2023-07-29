/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.7-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.protoype.lastIndexOf";

    var TEST_STRING = new String( " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~" );

//     writeLineToLog( "TEST_STRING = new String(\" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\")" );

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var j = 0;

    for ( k = 0, i = 0x0021; i < 0x007e; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.lastIndexOf(" +String.fromCharCode(i)+ ", 0)",
                                 -1,
                                 TEST_STRING.lastIndexOf( String.fromCharCode(i), 0 ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007e; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.lastIndexOf("+String.fromCharCode(i)+ ", "+ k +")",
                                 k,
                                 TEST_STRING.lastIndexOf( String.fromCharCode(i), k ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007e; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.lastIndexOf("+String.fromCharCode(i)+ ", "+k+1+")",
                                 k,
                                 TEST_STRING.lastIndexOf( String.fromCharCode(i), k+1 ) );
    }

    for ( k = 9, i = 0x0021; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 

                                 "String.lastIndexOf("+(String.fromCharCode(i) +
                                                    String.fromCharCode(i+1)+
                                                    String.fromCharCode(i+2)) +", "+ 0 + ")",
                                 LastIndexOf( TEST_STRING, String.fromCharCode(i) +
                                 String.fromCharCode(i+1)+String.fromCharCode(i+2), 0),
                                 TEST_STRING.lastIndexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                      0 ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.lastIndexOf("+(String.fromCharCode(i) +
                                                    String.fromCharCode(i+1)+
                                                    String.fromCharCode(i+2)) +", "+ k +")",
                                 k,
                                 TEST_STRING.lastIndexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                       k ) );
    }
    for ( k = 0, i = 0x0020; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.lastIndexOf("+(String.fromCharCode(i) +
                                                    String.fromCharCode(i+1)+
                                                    String.fromCharCode(i+2)) +", "+ k+1 +")",
                                 k,
                                 TEST_STRING.lastIndexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                       k+1 ) );
    }
    for ( k = 0, i = 0x0020; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.lastIndexOf("+
                                            (String.fromCharCode(i) +
                                            String.fromCharCode(i+1)+
                                            String.fromCharCode(i+2)) +", "+ (k-1) +")",
                                 LastIndexOf( TEST_STRING, String.fromCharCode(i) +
                                 String.fromCharCode(i+1)+String.fromCharCode(i+2), k-1),
                                 TEST_STRING.lastIndexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                       k-1 ) );
    }

    array[j++] = Assert.expectEq(   "String.lastIndexOf(" +TEST_STRING + ", 0 )", 0, TEST_STRING.lastIndexOf( TEST_STRING, 0 ) );
//    array[j++] = Assert.expectEq(   "String.lastIndexOf(" +TEST_STRING + ", 1 )", 0, TEST_STRING.lastIndexOf( TEST_STRING, 1 ));
    array[j++] = Assert.expectEq(   "String.lastIndexOf(" +TEST_STRING + ")", 0, TEST_STRING.lastIndexOf( TEST_STRING ));

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

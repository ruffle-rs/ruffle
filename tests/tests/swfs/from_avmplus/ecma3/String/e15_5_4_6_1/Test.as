/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.6-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.protoype.indexOf";

    var TEST_STRING = new String( " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~" );

//     writeLineToLog( "TEST_STRING = new String(\" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\")" );

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var j = 0;

    for ( k = 0, i = 0x0020; i < 0x007e; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.indexOf(" +String.fromCharCode(i)+ ", 0)",
                                 k,
                                 TEST_STRING.indexOf( String.fromCharCode(i), 0 ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007e; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.indexOf("+String.fromCharCode(i)+ ", "+ k +")",
                                 k,
                                 TEST_STRING.indexOf( String.fromCharCode(i), k ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007e; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.indexOf("+String.fromCharCode(i)+ ", "+k+1+")",
                                 -1,
                                 TEST_STRING.indexOf( String.fromCharCode(i), k+1 ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.indexOf("+(String.fromCharCode(i) +
                                                    String.fromCharCode(i+1)+
                                                    String.fromCharCode(i+2)) +", "+0+")",
                                 k,
                                 TEST_STRING.indexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                      0 ) );
    }

    for ( k = 0, i = 0x0020; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.indexOf("+(String.fromCharCode(i) +
                                                    String.fromCharCode(i+1)+
                                                    String.fromCharCode(i+2)) +", "+ k +")",
                                 k,
                                 TEST_STRING.indexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                       k ) );
    }
    for ( k = 0, i = 0x0020; i < 0x007d; i++, j++, k++ ) {
        array[j] = Assert.expectEq( 
                                 "String.indexOf("+(String.fromCharCode(i) +
                                                    String.fromCharCode(i+1)+
                                                    String.fromCharCode(i+2)) +", "+ k+1 +")",
                                 -1,
                                 TEST_STRING.indexOf( (String.fromCharCode(i)+
                                                       String.fromCharCode(i+1)+
                                                       String.fromCharCode(i+2)),
                                                       k+1 ) );
    }

    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING + ", 0 )", 0, TEST_STRING.indexOf( TEST_STRING, 0 ) );
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING + ", 1 )", -1, TEST_STRING.indexOf( TEST_STRING, 1 ));
   
    var TEST_STRING2 = " ";
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING2 + ", 0 )", 0, TEST_STRING.indexOf( TEST_STRING2, 0 ));
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING2 + ", 1 )", -1, TEST_STRING.indexOf( TEST_STRING2, 1 ));
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING2 + ", 1 )", 0, TEST_STRING.indexOf( TEST_STRING2));
   
    var TEST_STRING3:String = "abc def";
   
    for (i=0;i<7;i++){
        array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING3 + ", i )", i, TEST_STRING3.indexOf( TEST_STRING3.charAt(i)));
    }
   
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING3 + ", 8 )",0, TEST_STRING3.indexOf( TEST_STRING3.charAt(8)));
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING3+ " )",-1, TEST_STRING3.indexOf());
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING3+ " )",3, TEST_STRING3.indexOf(" "));
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING3+ " )",-1, TEST_STRING3.indexOf(null));
    array[j++] = Assert.expectEq(   "String.indexOf(" +TEST_STRING3+ " )",-1, TEST_STRING3.indexOf(null,0));


    return array;
}

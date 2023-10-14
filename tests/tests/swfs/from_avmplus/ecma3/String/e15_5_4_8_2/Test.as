/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.8-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.split";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    // case where separator is the empty string.

    var TEST_STRING = "this is a string object";

    array[item++] = Assert.expectEq(   
                                    "var s = new String( "+ TEST_STRING +" ); s.split('').length",
                                    TEST_STRING.length,
                                    (s = new String( TEST_STRING ), s.split('').length ) );

    for ( var i = 0; i < TEST_STRING.length; i++ ) {

        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split('')["+i+"]",
                                    TEST_STRING.charAt(i),
                                    (s = new String( TEST_STRING ), s.split('')[i]) );
    }

    // case where the value of the separator is undefined.  in this case. the value of the separator
    // should be ToString( separator ), or "undefined".

    var TEST_STRING = "thisundefinedisundefinedaundefinedstringundefinedobject";
    var EXPECT_STRING = new Array( "this", "is", "a", "string", "object" );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( "+ TEST_STRING +" ); s.split(void 0).length",
                                    EXPECT_STRING.length,
                                    (s = new String( TEST_STRING ), s.split(void 0).length ) );

    for ( var i = 0; i < EXPECT_STRING.length; i++ ) {
        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split(void 0)["+i+"]",
                                    EXPECT_STRING[i],
                                    (s = new String( TEST_STRING ), s.split(void 0)[i]) );
    }

    // case where the value of the separator is null.  in this case the value of the separator is "null".
    TEST_STRING = "thisnullisnullanullstringnullobject";
    var EXPECT_STRING = new Array( "this", "is", "a", "string", "object" );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( "+ TEST_STRING +" ); s.split(null).length",
                                    EXPECT_STRING.length,
                                    (s = new String( TEST_STRING ), s.split(null).length ) );

    for ( var i = 0; i < EXPECT_STRING.length; i++ ) {
        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split(null)["+i+"]",
                                    EXPECT_STRING[i],
                                    (s = new String( TEST_STRING ), s.split(null)[i]) );
    }

    // case where the value of the separator is a boolean.
    TEST_STRING = "thistrueistrueatruestringtrueobject";
    var EXPECT_STRING = new Array( "this", "is", "a", "string", "object" );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( "+ TEST_STRING +" ); s.split(true).length",
                                    EXPECT_STRING.length,
                                    (s = new String( TEST_STRING ), s.split(true).length ) );

    for ( var i = 0; i < EXPECT_STRING.length; i++ ) {
        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split(true)["+i+"]",
                                    EXPECT_STRING[i],
                                    (s = new String( TEST_STRING ), s.split(true)[i]) );
    }

    // case where the value of the separator is a number
    TEST_STRING = "this123is123a123string123object";
    var EXPECT_STRING = new Array( "this", "is", "a", "string", "object" );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( "+ TEST_STRING +" ); s.split(123).length",
                                    EXPECT_STRING.length,
                                    (s = new String( TEST_STRING ), s.split(123).length ) );

    for ( var i = 0; i < EXPECT_STRING.length; i++ ) {
        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split(123)["+i+"]",
                                    EXPECT_STRING[i],
                                    (s = new String( TEST_STRING ), s.split(123)[i]) );
    }


    // case where the value of the separator is a number
    TEST_STRING = "this123is123a123string123object";
    var EXPECT_STRING = new Array( "this", "is", "a", "string", "object" );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( "+ TEST_STRING +" ); s.split(123).length",
                                    EXPECT_STRING.length,
                                    (s = new String( TEST_STRING ), s.split(123).length ) );

    for ( var i = 0; i < EXPECT_STRING.length; i++ ) {
        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split(123)["+i+"]",
                                    EXPECT_STRING[i],
                                    (s = new String( TEST_STRING ), s.split(123)[i]) );
    }

    // case where the separator is not in the string
    TEST_STRING = "this is a string";
    EXPECT_STRING = new Array( "this is a string" );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( " + TEST_STRING + " ); s.split(':').length",
                                    1,
                                    (s = new String( TEST_STRING ), s.split(':').length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( " + TEST_STRING + " ); s.split(':')[0]",
                                    TEST_STRING,
                                    (s = new String( TEST_STRING ), s.split(':')[0] ) );

    // case where part but not all of separator is in the string.
    TEST_STRING = "this is a string";
    EXPECT_STRING = new Array( "this is a string" );
    array[item++] = Assert.expectEq(   
                                    "var s = new String( " + TEST_STRING + " ); s.split('strings').length",
                                    1,
                                    (s = new String( TEST_STRING ), s.split('strings').length ) );

    array[item++] = Assert.expectEq(   
                                    "var s = new String( " + TEST_STRING + " ); s.split('strings')[0]",
                                    TEST_STRING,
                                    (s = new String( TEST_STRING ), s.split('strings')[0] ) );

    // case where the separator is at the end of the string
    TEST_STRING = "this is a string";
    EXPECT_STRING = new Array( "this is a " );
    array[item++] = Assert.expectEq(   
                                    "var s = new String( " + TEST_STRING + " ); s.split('string').length",
                                    2,
                                    (s = new String( TEST_STRING ), s.split('string').length ) );

    for ( var i = 0; i < EXPECT_STRING.length; i++ ) {
        array[item++] = Assert.expectEq(   
                                    "var s = new String( "+TEST_STRING+" ); s.split('string')["+i+"]",
                                    EXPECT_STRING[i],
                                    (s = new String( TEST_STRING ), s.split('string')[i]) );
    }
    return array;
}


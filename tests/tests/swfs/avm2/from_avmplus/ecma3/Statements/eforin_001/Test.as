/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "forin-001";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The for...in  statement";
    var BUGNUMBER="330890";
    var BUGNUMBER="http://scopus.mcom.com/bugsplat/show_bug.cgi?id=344855";


    var tc = 0;
    var testcases = new Array();

    ForIn_1( { length:4, company:"netscape", year:2000, 0:"zero" } );
    ForIn_2( { length:4, company:"netscape", year:2000, 0:"zero" } );
    ForIn_3( { length:4, company:"netscape", year:2000, 0:"zero" } );

//    ForIn_6({ length:4, company:"netscape", year:2000, 0:"zero" });
    ForIn_8({ length:4, company:"netscape", year:2000, 0:"zero" });


    function ForIn_1( object ) {
        PropertyArray = new Array();
        ValueArray = new Array();

        for ( PropertyArray[PropertyArray.length] in object ) {
            ValueArray[ValueArray.length] =
                object[PropertyArray[PropertyArray.length-1]];
            
        }

        tcCompany = tc+0;
        tcLength = tc+1;
        tcZero = tc+2;
        tcYear = tc+3;

        // need a hack to make sure that the order of test cases
        // is constant... ecma stats that the order that for-in
        // is run does not have to be constant
        for ( var i = 0; i < PropertyArray.length; i++ ) {
                        
            switch( PropertyArray[i] ) {
                case "company":
                    testcases[tcCompany] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++;
                    break;

                case "length":
                    testcases[tcLength] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++;
                    break;

                case "year":
                    testcases[tcYear] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++;
                    break;

                case 0:
                    testcases[tcZero] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++;
                    break;

            }
        }

        testcases[tc++] = Assert.expectEq(
            
            "object.length",
            PropertyArray.length,
            object.length );
    }

    function ForIn_2( object ) {
        PropertyArray = new Array();
        ValueArray = new Array();
        var i = 0;

        for ( PropertyArray[i++] in object ) {
            ValueArray[ValueArray.length] =
                object[PropertyArray[PropertyArray.length-1]];
        }

        tcCompany = tc+0;
        tcLength = tc+1;
        tcZero = tc+2;
        tcYear = tc+3;

        // need a hack to make sure that the order of test cases
        // is constant... ecma stats that the order that for-in
        // is run does not have to be constant
        for ( var i = 0; i < PropertyArray.length; i++ ) {
            switch( PropertyArray[i] ) {
                case "company":
                    testcases[tcCompany] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++
                    break;

                case "length":
                    testcases[tcLength] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++
                    break;

                case "year":
                    testcases[tcYear] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++
                    break;

                case 0:
                    testcases[tcZero] = Assert.expectEq(
                        
                        "object[" + PropertyArray[i] +"]",
                        object[PropertyArray[i]],
                        ValueArray[i]
                    );
                    tc++
                    break;

            }
        }

        testcases[tc++] = Assert.expectEq(
            
            "object.length",
            PropertyArray.length,
            object.length );
    }

    function ForIn_3( object ) {
        var checkBreak = "pass";
        var properties = new Array();
        var values = new Array();

        for ( properties[properties.length] in object ) {
            values[values.length] = object[properties[properties.length-1]];
            break;
            checkBreak = "fail";
        }

        testcases[tc++] = Assert.expectEq(
            
            "check break out of for...in",
            "pass",
            checkBreak );

        testcases[tc++] = Assert.expectEq(
            
            "properties.length",
            1,
            properties.length );

        // we don't know which one of the properties
        // because we can't predict order
        var myTest = "PASSED";
        if( values[0] != object[properties[0]] )
            myTest = "FAILED";

        testcases[tc++] = Assert.expectEq(
            
            "object[properties[0]] == values[0]",
            "PASSED",
            myTest );
    }

    function ForIn_4( object ) {
        var result1 = 0;
        var result2 = 0;
        var result3 = 0;
        var result4 = 0;
        var i = 0;
        var property = new Array();

        butterbean: {
            result1++;

            for ( property[i++] in object ) {
                result2++;
                break;
                result4++;
            }
            result3++;
        }

        testcases[tc++] = Assert.expectEq(
            
            "verify labeled statement is only executed once",
            true,
            result1 == 1 );

        testcases[tc++] = Assert.expectEq(
            
            "verify statements in for loop are evaluated",
            true,
            result2 == i );

        testcases[tc++] = Assert.expectEq(
            
            "verify break out of labeled for...in loop",
            true,
            result4 == 0 );

        testcases[tc++] = Assert.expectEq(
            
            "verify break out of labeled block",
            true,
            result3 == 0 );
    }

    function ForIn_5 (object) {
        var result1 = 0;
        var result2 = 0;
        var result3 = 0;
        var result4 = 0;
        var i = 0;
        var property = new Array();

        bigredbird: {
            result1++;
            for ( property[i++] in object ) {
                result2++;
                break bigredbird;
                result4++;
            }
            result3++;
        }

        testcases[tc++] = Assert.expectEq(
            
            "verify labeled statement is only executed once",
            true,
            result1 == 1 );

        testcases[tc++] = Assert.expectEq(
            
            "verify statements in for loop are evaluated",
            true,
            result2 == i );

        testcases[tc++] = Assert.expectEq(
            
            "verify break out of labeled for...in loop",
            true,
            result4 == 0 );

        testcases[tc++] = Assert.expectEq(
            
            "verify break out of labeled block",
            true,
            result3 == 0 );
    }


    function ForIn_8( object ) {
        var checkBreak = "pass";
        var properties = new Array();
        var values = new Array();

        for ( properties[properties.length] in object ) {
            values[values.length] = object[properties[properties.length-1]];
            break;
            checkBreak = "fail";
        }

        testcases[tc++] = Assert.expectEq(
            
            "check break out of for...in",
            "pass",
            checkBreak );

        testcases[tc++] = Assert.expectEq(
            
            "properties.length",
            1,
            properties.length );

        // we don't know which one of the properties
        // because we can't predict order
        var myTest = "PASSED";
        if( values[0] != object[properties[0]] )
            myTest = "FAILED";


        testcases[tc++] = Assert.expectEq(
            
            "object[properties[0]] == object[properties[0]]",
            "PASSED",
            myTest );
    }


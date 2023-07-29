/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The String Constructor Called as a Function";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "String('string primitive')",   "string primitive", String('string primitive') );
    array[item++] = Assert.expectEq(   "String(void 0)",       "undefined",        String( void 0) );
    array[item++] = Assert.expectEq(   "String(null)","null",String(null));
    array[item++] = Assert.expectEq(   "String(true)",             "true",         String( true) );
    array[item++] = Assert.expectEq(   "String(false)",            "false",        String( false ) );
    array[item++] = Assert.expectEq(   "String(Boolean(true))",    "true",         String(Boolean(true)) );
    array[item++] = Assert.expectEq(   "String(Boolean(false))",   "false",        String(Boolean(false)) );
    array[item++] = Assert.expectEq(   "String(Boolean())",        "false",        String(Boolean(false)) );
    array[item++] = Assert.expectEq(   "String(new Array())",      "",             String( new Array()) );
    array[item++] = Assert.expectEq(   "String(new Array(1,2,3))", "1,2,3",        String( new Array(1,2,3)) );


    array[item++] = Assert.expectEq(     "String( Number.NaN )",       "NaN",                  String( Number.NaN ) );
    array[item++] = Assert.expectEq(     "String( 0 )",                "0",                    String( 0 ) );
    array[item++] = Assert.expectEq(     "String( -0 )",               "0",                   String( -0 ) );
    array[item++] = Assert.expectEq(     "String( Number.POSITIVE_INFINITY )", "Infinity",     String( Number.POSITIVE_INFINITY ) );
    array[item++] = Assert.expectEq(     "String( Number.NEGATIVE_INFINITY )", "-Infinity",    String( Number.NEGATIVE_INFINITY ) );
    array[item++] = Assert.expectEq(     "String( -1 )",               "-1",                   String( -1 ) );

    // cases in step 6:  integers  1e21 > x >= 1 or -1 >= x > -1e21

    array[item++] = Assert.expectEq(     "String( 1 )",                    "1",                    String( 1 ) );
    array[item++] = Assert.expectEq(     "String( 10 )",                   "10",                   String( 10 ) );
    array[item++] = Assert.expectEq(     "String( 100 )",                  "100",                  String( 100 ) );
    array[item++] = Assert.expectEq(     "String( 1000 )",                 "1000",                 String( 1000 ) );
    array[item++] = Assert.expectEq(     "String( 10000 )",                "10000",                String( 10000 ) );
    array[item++] = Assert.expectEq(     "String( 10000000000 )",          "10000000000",          String( 10000000000 ) );
    array[item++] = Assert.expectEq(     "String( 10000000000000000000 )", "10000000000000000000", String( 10000000000000000000 ) );
    array[item++] = Assert.expectEq(     "String( 100000000000000000000 )","100000000000000000000",String( 100000000000000000000 ) );

    array[item++] = Assert.expectEq(     "String( 12345 )",                    "12345",                    String( 12345 ) );
    array[item++] = Assert.expectEq(     "String( 1234567890 )",               "1234567890",               String( 1234567890 ) );

    array[item++] = Assert.expectEq(     "String( -1 )",                       "-1",                       String( -1 ) );
    array[item++] = Assert.expectEq(     "String( -10 )",                      "-10",                      String( -10 ) );
    array[item++] = Assert.expectEq(     "String( -100 )",                     "-100",                     String( -100 ) );
    array[item++] = Assert.expectEq(     "String( -1000 )",                    "-1000",                    String( -1000 ) );
    array[item++] = Assert.expectEq(     "String( -1000000000 )",              "-1000000000",              String( -1000000000 ) );
    array[item++] = Assert.expectEq(     "String( -1000000000000000 )",        "-1000000000000000",        String( -1000000000000000 ) );
    array[item++] = Assert.expectEq(     "String( -100000000000000000000 )",   "-100000000000000000000",   String( -100000000000000000000 ) );
    array[item++] = Assert.expectEq(     "String( -1000000000000000000000 )",  "-1e+21",                   String( -1000000000000000000000 ) );

    array[item++] = Assert.expectEq(     "String( -12345 )",                    "-12345",                  String( -12345 ) );
    array[item++] = Assert.expectEq(     "String( -1234567890 )",               "-1234567890",             String( -1234567890 ) );

    // cases in step 7: numbers with a fractional component, 1e21> x >1 or  -1 > x > -1e21,
    array[item++] = Assert.expectEq(     "String( 1.0000001 )",                "1.0000001",                String( 1.0000001 ) );


    // cases in step 8:  fractions between 1 > x > -1, exclusive of 0 and -0

    // cases in step 9:  numbers with 1 significant digit >= 1e+21 or <= 1e-6

    array[item++] = Assert.expectEq(     "String( 1000000000000000000000 )",   "1e+21",             String( 1000000000000000000000 ) );
    array[item++] = Assert.expectEq(     "String( 10000000000000000000000 )",   "1e+22",            String( 10000000000000000000000 ) );

    //  cases in step 10:  numbers with more than 1 significant digit >= 1e+21 or <= 1e-6
    array[item++] = Assert.expectEq(     "String( 1.2345 )",                    "1.2345",                  String( 1.2345));
    array[item++] = Assert.expectEq(     "String( 1.234567890 )",               "1.23456789",             String( 1.234567890 ));

    array[item++] = Assert.expectEq(     "String( .12345 )",                   "0.12345",               String(.12345 )     );
    array[item++] = Assert.expectEq(     "String( .012345 )",                  "0.012345",              String(.012345)     );
    array[item++] = Assert.expectEq(     "String( .0012345 )",                 "0.0012345",             String(.0012345)    );
    array[item++] = Assert.expectEq(     "String( .00012345 )",                "0.00012345",            String(.00012345)   );
    array[item++] = Assert.expectEq(     "String( .000012345 )",               "0.000012345",           String(.000012345)  );
    array[item++] = Assert.expectEq(     "String( .0000012345 )",              "0.0000012345",          String(.0000012345) );
    array[item++] = Assert.expectEq(     "String( .00000012345 )",             "1.2345e-7",            String(.00000012345));

    array[item++] = Assert.expectEq( "15.5.2 String()","",String() );

    return ( array );
}

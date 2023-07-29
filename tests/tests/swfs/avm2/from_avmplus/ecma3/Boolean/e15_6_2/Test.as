/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.6.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "15.6.2 The Boolean Constructor; 15.6.2.1 new Boolean( value ); 15.6.2.2 new Boolean()";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(    "typeof (new Boolean(1))",         "boolean",            typeof (new Boolean(1)) );
    array[item++] = Assert.expectEq(    "(new Boolean(1)).constructor",    Boolean.prototype.constructor,   (new Boolean(1)).constructor );

    var TESTBOOL:Boolean=new Boolean(1);
    array[item++] = Assert.expectEq( "TESTBOOL.toString()","true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(1)).valueOf()",true,         (new Boolean(1)).valueOf() );
    array[item++] = Assert.expectEq( "typeof new Boolean(1)","boolean",    typeof (new Boolean(1)) );
    array[item++] = Assert.expectEq(    "(new Boolean(0)).constructor",    Boolean.prototype.constructor,   (new Boolean(0)).constructor );

    var TESTBOOL:Boolean=new Boolean(0);
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()","false",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(0)).valueOf()",   false,       (new Boolean(0)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(0)","boolean",typeof (new Boolean(0)) );
    array[item++] = Assert.expectEq(    "(new Boolean(-1)).constructor",    Boolean.prototype.constructor,   (new Boolean(-1)).constructor );

    var TESTBOOL:Boolean=new Boolean(-1);
    array[item++] = Assert.expectEq( "TESTBOOL.toString()","true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(-1)).valueOf()",   true,       (new Boolean(-1)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(-1)",         "boolean",   typeof (new Boolean(-1)) );
    array[item++] = Assert.expectEq(    "(new Boolean('')).constructor",    Boolean.prototype.constructor,   (new Boolean('')).constructor );

    var TESTBOOL:Boolean=new Boolean("");
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()", "false",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean('')).valueOf()",   false,       (new Boolean("")).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean('')",         "boolean",   typeof (new Boolean("")) );
    array[item++] = Assert.expectEq(    "(new Boolean('1')).constructor",    Boolean.prototype.constructor,   (new Boolean("1")).constructor );

    var TESTBOOL:Boolean=new Boolean('1');
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()", "true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean('1')).valueOf()",   true,       (new Boolean('1')).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean('1')",         "boolean",   typeof (new Boolean('1')) );
    array[item++] = Assert.expectEq(    "(new Boolean('0')).constructor",    Boolean.prototype.constructor,   (new Boolean('0')).constructor );

    var TESTBOOL:Boolean=new Boolean('0');
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()", "true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean('0')).valueOf()",   true,       (new Boolean('0')).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean('0')",         "boolean",   typeof (new Boolean('0')) );
    array[item++] = Assert.expectEq(    "(new Boolean('-1')).constructor",    Boolean.prototype.constructor,   (new Boolean('-1')).constructor );

    var TESTBOOL:Boolean=new Boolean('-1');
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()","true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean('-1')).valueOf()",   true,       (new Boolean('-1')).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean('-1')",         "boolean",   typeof (new Boolean('-1')) );
    array[item++] = Assert.expectEq(    "(new Boolean(new Boolean(true))).constructor",    Boolean.prototype.constructor,   (new Boolean(new Boolean(true))).constructor );

    var TESTBOOL:Boolean=new Boolean(new Boolean(true));
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()", "true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(new Boolean(true))).valueOf()",   true,       (new Boolean(new Boolean(true))).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(new Boolean(true))",         "boolean",   typeof (new Boolean(new Boolean(true))) );
    array[item++] = Assert.expectEq(    "(new Boolean(Number.NaN)).constructor",    Boolean.prototype.constructor,   (new Boolean(Number.NaN)).constructor );

    var TESTBOOL:Boolean=new Boolean(Number.NaN);
    array[item++] = Assert.expectEq( "TESTBOOL.toString()","false",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(Number.NaN)).valueOf()",   false,       (new Boolean(Number.NaN)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(Number.NaN)",         "boolean",   typeof (new Boolean(Number.NaN)) );
    array[item++] = Assert.expectEq(    "(new Boolean(null)).constructor",    Boolean.prototype.constructor,   (new Boolean(null)).constructor );

    var TESTBOOL:Boolean=new Boolean(null);
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()","false",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(null)).valueOf()",   false,       (new Boolean(null)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(null)",         "boolean",   typeof (new Boolean(null)) );
    array[item++] = Assert.expectEq(    "(new Boolean(void 0)).constructor",    Boolean.prototype.constructor,   (new Boolean(void 0)).constructor );

    var TESTBOOL:Boolean=new Boolean(void 0);
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()","false",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(void 0)).valueOf()",   false,       (new Boolean(void 0)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(void 0)",         "boolean",   typeof (new Boolean(void 0)) );
    array[item++] = Assert.expectEq(    "(new Boolean(Number.POSITIVE_INFINITY)).constructor",    Boolean.prototype.constructor,   (new Boolean(Number.POSITIVE_INFINITY)).constructor );

    var TESTBOOL:Boolean=new Boolean(Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()", "true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(Number.POSITIVE_INFINITY)).valueOf()",   true,       (new Boolean(Number.POSITIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(Number.POSITIVE_INFINITY)",         "boolean",   typeof (new Boolean(Number.POSITIVE_INFINITY)) );
    array[item++] = Assert.expectEq(    "(new Boolean(Number.NEGATIVE_INFINITY)).constructor",    Boolean.prototype.constructor,   (new Boolean(Number.NEGATIVE_INFINITY)).constructor );

    var TESTBOOL:Boolean=new Boolean(Number.NEGATIVE_INFINITY);
    array[item++] = Assert.expectEq(  "TESTBOOL.toString()", "true",TESTBOOL.toString());
    array[item++] = Assert.expectEq(    "(new Boolean(Number.NEGATIVE_INFINITY)).valueOf()",   true,       (new Boolean(Number.NEGATIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(    "typeof new Boolean(Number.NEGATIVE_INFINITY)",         "boolean",   typeof (new Boolean(Number.NEGATIVE_INFINITY) ));
    array[item++] = Assert.expectEq(    "(new Boolean(Number.NEGATIVE_INFINITY)).constructor",    Boolean.prototype.constructor,   (new Boolean(Number.NEGATIVE_INFINITY)).constructor );

//TO-DO: Removing " , " from the Assert.expectEq to make it pass 3 arguments instead of 4 arguments e.g.
    //   array[item++] = Assert.expectEq( "15.6.2.2",   "typeof new Boolean()",        "boolean",    typeof (new Boolean()) ); 
    // is changed to 
    //    array[item++] = Assert.expectEq( "15.6.2.2 typeof new Boolean()",        "boolean",    typeof (new Boolean()) );
    var TESTBOOL:Boolean=new Boolean();
    array[item++] = Assert.expectEq( "15.6.2.2 TESTBOOL.toString()","false",TESTBOOL.toString());
    array[item++] = Assert.expectEq( "15.6.2.2 (new Boolean()).valueOf()",   false,       (new Boolean()).valueOf() );
    array[item++] = Assert.expectEq( "15.6.2.2 typeof new Boolean()",        "boolean",    typeof (new Boolean()) );

    return ( array );
}

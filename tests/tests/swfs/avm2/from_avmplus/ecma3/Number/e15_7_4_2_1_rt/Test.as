/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.4.2-1";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var thisError="no error thrown";
    try{
        thisError = Number.prototype.toString();
    }
    catch(e)
    {
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq( "Number.prototype.toString()",
                                              "0",
                                              thisError);
    }

    array[item++] = Assert.expectEq( "typeof(Number.prototype.toString())", "string",      typeof(Number.prototype.toString()) );


    var expectedError = 1056;
    //TO-DO: commenting as3Enabled
    //if (as3Enabled) {
        expectedError = 1037;
    //}

    thisError="no error thrown";
    try{
        s = Number.prototype.toString;
        o = new Number();
        o.toString = s;
    }
    catch(e1)
    {
        thisError=e1.toString();
    }finally{
        array[item++] = Assert.expectEq( "s = Number.prototype.toString",
                                            Utils.REFERENCEERROR+expectedError,
                                            Utils.referenceError(thisError));
    }

    thisError="no error thrown";
    try{
        s = Number.prototype.toString;
        o = new Number(1);
        o.toString = s;
    }
    catch(e2){
        thisError=e2.toString();
    }finally{
        array[item++] = Assert.expectEq( "s = Number.prototype.toString",
                                                Utils.REFERENCEERROR+expectedError,
                                                Utils.referenceError(thisError));
    }

    thisError="no error thrown";
    try{
        s = Number.prototype.toString;
        o = new Number(-1);
        o.toString = s;
    }
    catch(e3){
        thisError=e3.toString();
    }finally{
        array[item++] = Assert.expectEq( "s = Number.prototype.toString",
                                            Utils.REFERENCEERROR+expectedError,
                                            Utils.referenceError(thisError));
    }

    var MYNUM = new Number(255);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(255); MYNUM.toString(10)",          "255",      MYNUM.toString(10) );

    var MYNUM = new Number(Number.NaN);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(Number.NaN); MYNUM.toString(10)",   "NaN",      MYNUM.toString(10) );

    var MYNUM = new Number(Infinity);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(Infinity); MYNUM.toString(10)",   "Infinity",   MYNUM.toString(10) );

    var MYNUM = new Number(-Infinity);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(-Infinity); MYNUM.toString(10)",   "-Infinity", MYNUM.toString(10) );

    return ( array );
}

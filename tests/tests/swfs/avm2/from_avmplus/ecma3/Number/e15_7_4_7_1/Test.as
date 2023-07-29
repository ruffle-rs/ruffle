/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
 * Date: 2001-07-15
 *
 * SUMMARY: Testing Number.prototype.toPrecision(precision)
 * See EMCA 262 Edition 3 Section 15.7.4.7
 *
 */
//-----------------------------------------------------------------------------
//     var SECTION = "15.7.4.6-1";
//     var VERSION = "";
//     var TITLE = "Testing Number.prototype.toPrecision(precision)";
//     var bug = '(none)';
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var UBound = 0;
    var cnIsRangeError = 'instanceof RangeError';
    var cnNotRangeError = 'NOT instanceof RangeError';
    var cnNoErrorCaught = 'NO ERROR CAUGHT...';
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];
    var testNum:Number = 5.123456;
    var testNum2 = NaN
    var testNum3 = Number.POSITIVE_INFINITY
    var testNum4 = 0;
    var testNum5 = 4000;


    status = 'Section A of test: no error intended!';
    actual = testNum.toPrecision(4);
    expect = '5.123';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);


    status = 'Section B of test: no error intended!';
    actual = testNum.toPrecision(undefined);
    expect = testNum.toString();
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);




    status = 'Section C of test: no error intended!';
    actual = Number.prototype.toPrecision.length
    expect = 1
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section D of test: no error intended!';
    actual = testNum2.toPrecision(6);
    expect = "NaN"
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section E of test: no error intended!';
    actual = testNum3.toPrecision(6);
    expect = "Infinity"
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    var thisError:String="no error";
    try{
        testNum.toPrecision(0)
    }catch(e:RangeError){
        thisError=e.toString();
    }

    status = 'Section F of test: error intended!';
    actual = Utils.rangeError(thisError);
    expect = "RangeError: Error #1002"
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);


    status = 'Section G of test: no error intended!';
    actual = testNum.toPrecision(21);
    expect = "5.12345600000000001017"
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    var thisError:String="no error";
    try{
        testNum.toPrecision(-1)
    }catch(e:RangeError){
        thisError=e.toString();
    }

    status = 'Section H of test: error intended!';
    actual = Utils.rangeError(thisError);
    expect = "RangeError: Error #1002"
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    var thisError:String="no error";
    try{
        testNum.toPrecision(22)
    }catch(e:RangeError){
        thisError=e.toString();
    }

    status = 'Section I of test: error intended!';
    actual = Utils.rangeError(thisError);
    expect = "RangeError: Error #1002"
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section J of test: no error intended!';
    actual = testNum4.toPrecision(4);
    expect = '0.0000';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section K of test: no error intended!';
    actual = testNum5.toPrecision(3);
    expect = '4.00e+3';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    // Regression tests

    // Bugzilla 442974

    // status = 'Section R-1 of test: no error intended!';
    // actual = Number.MIN_VALUE.toPrecision(21);
    // expect = '4.94065645841246544177e-324';
    // array[item++] = Assert.expectEq( status, expect, actual);


///////////////////////////    OOPS....    ///////////////////////////////

    return array;
}

/*
function captureThis()
{
  statusitems[UBound] = status;
  actualvalues[UBound] = actual;
  expectedvalues[UBound] = expect;
  UBound++;
}
 */

/*
function test()
{
  enterFunc ('test');
  printBugNumber (bug);
  printStatus (summary);
 
  for (var i = 0; i < UBound; i++)
  {
    reportCompare(expectedvalues[i], actualvalues[i], statusitems[i]);
  }

  exitFunc ('test');
}
 */

function catchError(sEval)
{
    try {eval(sEval);}
    catch(e) {return isRangeError(e);}
    return cnNoErrorCaught;
}


function isRangeError(obj)
{
    if (obj instanceof RangeError)
        return cnIsRangeError;
    return cnNotRangeError;
}

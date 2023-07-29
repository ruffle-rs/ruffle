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
 * SUMMARY: Testing Number.prototype.toExponential(fractionDigits)
 * See EMCA 262 Edition 3 Section 15.7.4.6
 *
 */
//-----------------------------------------------------------------------------
//     var SECTION = "15.7.4.6-1";
//     var VERSION = "";
//     var TITLE = "Testing Number.prototype.toExponential(fractionDigits)";
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
    var testNum = 77.1234;
    var testNum2=NaN;
    var testNum3=0;
    var testNum4=Number.POSITIVE_INFINITY;
    var testNum5=315003;


    status = 'Section A of test: no error intended!';
    actual = testNum.toExponential(4);
    expect = '7.7123e+1';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section B of test: no error intended!';
    actual = Number.prototype.toExponential.length
    expect = 1
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    // status = 'Section C of test: no error intended!';
    // actual = testNum.toExponential();
    // expect = '7.71234e+1';
    // //captureThis();
    // array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section D of test: no error intended!';
    actual = testNum.toExponential(5);
    expect = '7.71234e+1';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section E of test: no error intended!';
    actual = testNum2.toExponential(5);
    expect = 'NaN';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section F of test: no error intended!';
    actual = testNum3.toExponential(5);
    expect = '0.00000e-16';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section G of test: no error intended!';
    actual = testNum4.toExponential(4);
    expect = 'Infinity';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    var thisError:String="no error";
    try{
        testNum.toExponential(-1);
    }catch(e:RangeError){
        thisError=e.toString()
    }

    status = 'Section H of test: error intended!';
    actual = Utils.rangeError(thisError);
    expect = "RangeError: Error #1002";
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    var thisError:String="no error";
    try{
        testNum.toExponential(21);
    }catch(e:RangeError){
        thisError=e.toString()
    }

    status = 'Section I of test: error intended!';
    actual = Utils.rangeError(thisError);
    expect = "RangeError: Error #1002";
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section J of test: no error intended!';
    actual = testNum5.toExponential(2);
    expect = '3.15e+5';
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);


    // Regression tests

    // Bugzilla 513039

    status = 'Section R-1 of test: no error intended!';
    actual = (-0.1).toFixed(0);
    expect = '-0'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-2 of test: no error intended!';
    actual = (-0.05).toFixed(0);
    expect = '-0'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-3 of test: no error intended!';
    actual = (0).toFixed(0);
    expect = '0'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-4 of test: no error intended!';
    actual = (0.05).toFixed(0);
    expect = '0'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-5 of test: no error intended!';
    actual = (0.1).toFixed(0);
    expect = '0'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-6 of test: no error intended!';
    actual = (0.00005).toFixed(2);
    expect = '0.00'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-7 of test: no error intended!';
    actual = (0.00007).toFixed(2);
    expect = '0.00'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-8 of test: no error intended!';
    actual = (0.00009).toFixed(2);
    expect = '0.00'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-9 of test: no error intended!';
    actual = (5e-7).toFixed(2);
    expect = '0.00'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-10 of test: no error intended!';
    actual = (7e-7).toFixed(2);
    expect = '0.00'
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = 'Section R-11 of test: no error intended!';
    actual = (9e-7).toFixed(2);
    expect = '0.00'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-12 of test: no error intended!';
    actual = (0.00005).toFixed(3);
    expect = '0.000'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-13 of test: no error intended!';
    actual = (0.00007).toFixed(3);
    expect = '0.000'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-14 of test: no error intended!';
    actual = (0.00009).toFixed(3);
    expect = '0.000'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-15 of test: no error intended!';
    actual = (5e-7).toFixed(3);
    expect = '0.000'
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Section R-16 of test: no error intended!';
    actual = (7e-7).toFixed(3);
    expect = '0.000'
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = 'Section R-17 of test: no error intended!';
    actual = (9e-7).toFixed(3);
    expect = '0.000'
    array[item++] = Assert.expectEq( status, expect, actual);

    // Bugzilla 478796

    // status = 'Section R-18 of test: no error intended!';
    // actual = (1000000000000000128).toFixed(0);
    // expect = '1000000000000000128'
    // array[item++] = Assert.expectEq( status, expect, actual);

    // status = 'Section R-19 of test: no error intended!';
    // actual = (1000000000000000128).toFixed(1);
    // expect = '1000000000000000128.0'
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

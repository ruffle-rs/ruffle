/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
*
* Date:    21 May 2002
* SUMMARY: ECMA conformance of Function.prototype.apply
*
*   Function.prototype.apply(thisArg, argArray)
*
* See ECMA-262 Edition 3 Final, Section 15.3.4.3
*/
//-----------------------------------------------------------------------------
//     var SECTION = "e15_3_4_3_1";
//     var VERSION = "";

//     var TITLE   = "Testing ECMA conformance of Function.prototype.apply";
//     var bug = 145791;
import com.adobe.test.Assert;

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var UBound = 0;
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];


    function F0(a)
    {
      return "" + this + arguments.length;
    }
    
    function F1(a)
    {
      return "" + this + a;
    }
    
    function F2()
    {
      return "" + this;
    }



    /*
     * Function.prototype.apply.length should return 2
     */

     //TO-DO: replacing inSection(1) with "function string1"
    //status = inSection(1);
    status = "function string1"
    actual = Function.prototype.apply.length;
    expect = 2;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    /*
     * When |thisArg| is not provided to the apply() method, the
     * called function must be passed the global object as |this|
     */
   status = "function string2";
    actual = F0.apply();
    expect = "" + this + 0;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    /*
     * If |argArray| is not provided to the apply() method, the
     * called function should be invoked with an empty argument list
     */
    status = "function string3";
    actual = F0.apply("");
    expect = "" + "" + 0;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string4";
    actual = F0.apply(true);
    expect = "" + true + 0;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    /*
     * Function.prototype.apply(x) and
     * Function.prototype.apply(x, undefined) should return the same result
     */
    status = "function string5"
    actual = F1.apply(0, undefined);
    expect = F1.apply(0);
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string6";
    actual = F1.apply("", undefined);
    expect = F1.apply("");
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string7";
    actual = F1.apply(null, undefined);
    expect = F1.apply(null);
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string8";
    actual = F1.apply(undefined, undefined);
    expect = F1.apply(undefined);
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    /*
     * Function.prototype.apply(x) and
     * Function.prototype.apply(x, null) should return the same result
     */
   status = "function string9";
    actual = F1.apply(0, null);
    expect = F1.apply(0);
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string10";
    actual = F1.apply("", null);
    expect = F1.apply("");
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string11"
    actual = F1.apply(null, null);
    expect = F1.apply(null);
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = "function string12";
    actual = F1.apply(undefined, null);
    expect = F1.apply(undefined);
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    /*
     * Function.prototype.apply() and
     * Function.prototype.apply(undefined) should return the same result
     */
    sstatus = "function string13";
    actual = F2.apply(undefined);
    expect = F2.apply();
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    /*
     * Function.prototype.apply() and
     * Function.prototype.apply(null) should return the same result
     */
    status = "function string14";
    actual = F2.apply(null);
    expect = F2.apply();
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    return array;
}


/*
function addThis()
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
  enterFunc('test');
  printBugNumber(bug);
  printStatus(summary);

  for (var i=0; i<UBound; i++)
  {
    reportCompare(expectedvalues[i], actualvalues[i], statusitems[i]);
  }

  exitFunc ('test');
}
*/

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 2001-08-27
 *
 * SUMMARY:  Testing binding of function names
 *
 * Brendan:
 *
 * "... the question is, does Rhino bind 'sum' in the global object
 * for the following test? If it does, it's buggy.
 *
 *   var f = function sum(){};
 *   print(sum);  // should fail with 'sum is not defined' "
 *
 */
//-----------------------------------------------------------------------------
//     var SECTION = "binding_001";
//     var VERSION = "";
    var ERR_REF_YES = 'ReferenceError';
    var ERR_REF_NO = 'did NOT generate a ReferenceError';
    var TITLE   = "Testing binding of function names";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var UBound = 0;
    var statusitems = [];
    var actualvalues = [];
    var expectedvalues = [];
    var status = TITLE;
    var actual = ERR_REF_NO;
    var expect= ERR_REF_YES;
    //var match;

    try
    {
      var f = function sum(){};
    
      trace(sum);
    }
    catch (e)
    {
      status = 'e instanceof ReferenceError';
      actual = e instanceof ReferenceError;
      expect = true;
      array[item++] = Assert.expectEq( status, isReferenceError(expect), isReferenceError(actual));
    
    
      /*
       * This test is more literal, and one day may not be valid.
       * Searching for literal string "ReferenceError" in e.toString()
       */
      status = 'e.toString().search(/ReferenceError/)';
      var match = e.toString().search(/ReferenceError/);
    
      actual = (match > -1);
    
    
      expect = true;
      array[item++] = Assert.expectEq( status, isReferenceError(expect), isReferenceError(actual));
    }
    return array;
}

// converts a Boolean result into a textual result -
function isReferenceError(bResult)
{
  return bResult? ERR_REF_YES : ERR_REF_NO;
}

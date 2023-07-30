/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert; 

// TODO: REVIEW AS4 CONVERSION ISSUE 

/*
 * Date: 13 August 2001
 *
 * SUMMARY: Invoking an undefined function should produce a ReferenceError
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=95101
 */
//-----------------------------------------------------------------------------
//     var SECTION = "regress_95101";
//     var VERSION = "";
//     var bug = 95101;
//     var TITLE   = "Invoking an undefined function should produce a ReferenceError";
    var msgERR_REF_YES = 'ReferenceError';
    var msgERR_REF_NO = 'did NOT generate a ReferenceError';

   // printBugNumber (bug);
   //  printStatus (TITLE);

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


    try
    {
      xxxyyyzzz();
    }
    catch (e)
    {
      status = 'Section 1 of test';
      actual = e instanceof ReferenceError;
      expect = true;
      array[item++] = Assert.expectEq( status, expect, actual);
    
    
      /*
       * This test is more literal, and may one day be invalid.
       * Searching for literal string "ReferenceError" in e.toString()
       */
      status = 'Section 2 of test';
      var match = e.toString().search(/ReferenceError/);
      actual = (match > -1);
      expect = true;
      array[item++] = Assert.expectEq( status, expect, actual);
    }
    return array;
}


// converts a Boolean result into a textual result -
function isReferenceError(bResult)
{
  return bResult? msgERR_REF_YES : msgERR_REF_NO;
}

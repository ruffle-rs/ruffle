/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 01 June 2001
 *
 * SUMMARY: Testing that we don't crash on switch case -1...
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=83532
 *
 */
//-------------------------------------------------------------------------------------------------
// var bug = 83532;
var summary = "Testing that we don't crash on switch case -1";



var testcases = getTestCases();

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

function getTestCases()
{
    var array = new Array();
    var item = 0;
    
    doTest();
    
    array[item++] = Assert.expectEq("Make sure we don't crash", true, true);
    
    return array;
}

function doTest()
{
 /* TO-DO: Commenting out the printBugNumber() and printStatus
  printBugNumber (bug);
  
  printStatus (summary);
*/
  // Just testing that we don't crash on these -
  function f () {switch(1) {case -1:}}
  function g(){switch(1){case (-1):}}
  var h = function() {switch(1) {case -1:}}
  f();
  g();
  h();
}

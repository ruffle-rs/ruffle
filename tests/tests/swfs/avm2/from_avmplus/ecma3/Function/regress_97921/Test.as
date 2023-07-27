/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 10 September 2001
 *
 * SUMMARY: Testing with() statement with nested functions
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=97921
 *
 * Brendan: "The bug is peculiar to functions that have formal parameters,
 * but that are called with fewer actual arguments than the declared number
 * of formal parameters."
 */
//-----------------------------------------------------------------------------
//     var SECTION = "";
//     var VERSION = "";
//     var TITLE = "Testing with() statement with nested functions";


//     var bug = 97921;


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var UBound = 0;
    var cnYES = 'Inner value === outer value';
    var cnNO = "Inner value !== outer value!";
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];
    var outerValue = '';
    var innerValue = '';
    var useWith = '';


    function F(i)
    {
      i = 0;
      if(useWith) with(1){i;}
      i++;
    
      outerValue = i; // capture value of i in outer function
      F1 = function() {innerValue = i;}; // capture value of i in inner function
      
      F1();
    }
    
    //TO-DO: replacing inSection(1) with "function string1"
    //status = inSection(1);
    status = "function string1";

    useWith=false;
    F(); // call F without supplying the argument
    actual = innerValue == outerValue;
    expect = true;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    //TO-DO: replacing inSection(1) with "function string1"
    //status = inSection(1);
    status = "function string2";
    useWith=true;
    F(); // call F without supplying the argument
    actual = innerValue == outerValue;
    
    expect = true;
    
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    
    function G(i)
    {
      i = 0;
      with (new Object()) {i=100};
      i++;
    
      outerValue = i; // capture value of i in outer function
      G1 = function() {innerValue = i;}; // capture value of i in inner function
      G1();
      
    }
    
    
    //TO-DO: replacing inSection(1) with "function string1"
    //status = inSection(1);
    status = "function string3";
    G(); // call G without supplying the argument
    actual = innerValue == 101;
    expect = true;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 4";
    G(); // call G without supplying the argument
    actual = innerValue == outerValue;
    expect = true;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    return array;
}

/*
function addThis()
{
  statusitems[UBound] = status;
  actualvalues[UBound] = areTheseEqual(actual);
  
  expectedvalues[UBound] = areTheseEqual(expect);
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

function areTheseEqual(yes)
{
  return yes? cnYES : cnNO
}

